//! Keyring 与 DB 一致性辅助。
//!
//! #规则
//!
//!1. **DB 先于 Keyring**：DB提交成功后,再写 Keyring。失败时回滚 DB 和已写入的 Keyring。
//!2. **删除反过来**：DB 删除先 commit,然后 best-effort清理 Keyring(失败仅 warn,不回滚 DB)。
//!3. **Keyring自身原子**：一组 keyring写入,任一失败回滚前面已写入的项。
//!
//! # 为什么
//!
//! - Keyring写入先于 DB写入 →失败时 DB 没变(干净),但 DB失败时 Keyring留下孤儿(下次启动看见 key 但查不到 record,语义混乱)。
//! - DB 先 commit 再写 Keyring → DB 是 source of truth,Keyring失败只 warn 不影响 DB(用户重试或人工补 Keyring)。
//!
//! # 用法
//!
//! ```ignore
//! let mut conn = state.db.lock()?;
//! let old_keys = keyring_db::run_tx(&mut conn, |tx| {
//! let old_keys = db::list_bindings_by_tool(tx, &tool_id)?
//! .into_iter().filter_map(|b| b.keyring_key).collect();
//! tx.execute("DELETE FROM tool_bindings WHERE tool_id = ?1", [&tool_id])?;
//! tx.execute("INSERT INTO tool_bindings ...", ...)?;
//! Ok(old_keys)
//! })?;
//! drop(conn);
//!
//! apply_config_for_tool(...)?; // 配置写失败时人工回滚 DB
//!
//! keyring_db::write_keyring(KEYRING_SERVICE, &[(binding_account(&id), api_key.clone())])?;
//! keyring_db::delete_keyring_best_effort(KEYRING_SERVICE, &old_keys);
//! ```

use crate::keyring_store;
use rusqlite::Connection;

/// 在 DB事务中执行 db_op,成功后 commit,失败自动回滚。
///
/// `db_op` 应只执行 DB 操作,不接触 OS / 网络 / 文件系统。
pub fn run_tx<F>(conn: &mut Connection, db_op: F) -> Result<(), String>
where
 F: FnOnce(&Connection) -> rusqlite::Result<()>,
{
 let tx = conn
 .transaction()
 .map_err(|e| format!("事务启动失败: {}", e))?;
 db_op(&tx).map_err(|e| format!("DB 操作失败: {}", e))?;
 tx.commit().map_err(|e| format!("事务提交失败: {}", e))
}

/// 在 DB事务中执行 db_op,可返回数据(用于"先读旧值再写新值"的场景)。
pub fn run_tx_with<F, T>(conn: &mut Connection, db_op: F) -> Result<T, String>
where
 F: FnOnce(&Connection) -> rusqlite::Result<T>,
{
 let tx = conn
 .transaction()
 .map_err(|e| format!("事务启动失败: {}", e))?;
 let value = db_op(&tx).map_err(|e| format!("DB 操作失败: {}", e))?;
 tx.commit()
 .map_err(|e| format!("事务提交失败: {}", e))?;
 Ok(value)
}

///写 Keyring 项。任一失败时回滚已写入的项并返回 Err。
///
/// `writes` 按顺序写入;前面的先写入,失败时倒序删除。
pub fn write_keyring(service: &str, writes: &[(String, String)]) -> Result<(), String> {
 let mut done: Vec<&str> = Vec::with_capacity(writes.len());
 for (account, key) in writes {
 if let Err(e) = keyring_store::set_key(service, account, key) {
 for prev in done.iter().rev() {
 let _ = keyring_store::delete_key(service, prev);
 }
 return Err(format!("Keyring写入失败 ({}): {}", account, e));
 }
 done.push(account.as_str());
 }
 Ok(())
}

///写单个 Keyring 项。失败时直接返回 Err,不涉及回滚(用于"DB 已提交,Keyring 是 best-effort"场景)。
///
/// 调用方负责决定:Keyring失败是否要回滚 DB。
pub fn write_keyring_one(service: &str, account: &str, key: &str) -> Result<(), String> {
 keyring_store::set_key(service, account, key)
 .map_err(|e| format!("Keyring写入失败 ({}): {}", account, e))
}

/// 删除 Keyring 项。失败仅记录 warning(用于"DB 已删除,Keyring 是 best-effort"场景)。
///
/// 不应回滚 DB,因为 DB 已删除/已变更,Keyring残留可在下次启动清理。
pub fn delete_keyring_best_effort(service: &str, accounts: &[String]) {
 for account in accounts {
 if let Err(e) = keyring_store::delete_key(service, account) {
 eprintln!("Warning: Keyring清理失败 ({}): {}", account, e);
 }
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_run_tx_commits_on_success() {
 // 用 in-memory SQLite验证事务行为
 let mut conn = Connection::open_in_memory().unwrap();
 conn.execute_batch(
 "CREATE TABLE t (id INTEGER PRIMARY KEY, v TEXT NOT NULL);",
 )
 .unwrap();
 conn.execute("INSERT INTO t (id, v) VALUES (1, 'old')", []).unwrap();

 run_tx(&mut conn, |tx| {
 tx.execute("UPDATE t SET v = 'new' WHERE id =1", [])?;
 Ok(())
 })
 .unwrap();

 let v: String = conn
 .query_row("SELECT v FROM t WHERE id =1", [], |r| r.get(0))
 .unwrap();
 assert_eq!(v, "new");
 }

 #[test]
 fn test_run_tx_rolls_back_on_error() {
 let mut conn = Connection::open_in_memory().unwrap();
 conn.execute_batch(
 "CREATE TABLE t (id INTEGER PRIMARY KEY, v TEXT NOT NULL UNIQUE);",
 )
 .unwrap();
 conn.execute("INSERT INTO t (id, v) VALUES (1, 'a')", []).unwrap();
 conn.execute("INSERT INTO t (id, v) VALUES (2, 'b')", []).unwrap();

 let result: Result<(), String> = run_tx(&mut conn, |tx| {
 tx.execute("UPDATE t SET v = 'c' WHERE id =1", [])?;
 // UNIQUE约束触发回滚
 tx.execute("UPDATE t SET v = 'c' WHERE id =2", [])?;
 Ok(())
 });
 assert!(result.is_err());

 // 两个值都应该保持原样
 let v1: String = conn
 .query_row("SELECT v FROM t WHERE id =1", [], |r| r.get(0))
 .unwrap();
 let v2: String = conn
 .query_row("SELECT v FROM t WHERE id =2", [], |r| r.get(0))
 .unwrap();
 assert_eq!(v1, "a");
 assert_eq!(v2, "b");
 }

 #[test]
 fn test_run_tx_with_returns_data() {
 let mut conn = Connection::open_in_memory().unwrap();
 conn.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, v TEXT);")
 .unwrap();
 conn.execute("INSERT INTO t (id, v) VALUES (1, 'x')", []).unwrap();

 let captured: Vec<String> = run_tx_with(&mut conn, |tx| {
 let mut stmt = tx.prepare("SELECT v FROM t WHERE id =1")?;
 let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
 let out: Result<Vec<String>, _> = rows.collect();
 out
 })
 .unwrap();

 assert_eq!(captured, vec!["x"]);
 }
}
