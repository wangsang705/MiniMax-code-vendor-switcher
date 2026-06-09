use super::{
 binding_account, fetch_provider_key, get_tool_lock, now_ts, provider_account,
 apply_config_for_tool, AppState, KEYRING_SERVICE,
};
use crate::db;
use crate::keyring_db;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct CreateProviderInput {
 pub id: String,
 pub name: String,
 pub api_base: String,
 pub anthropic_mode: bool,
 pub api_key: Option<String>,
}

fn provider_has_key(state: &State<AppState>, provider_id: &str) -> Result<bool, String> {
 let account = provider_account(provider_id);
 if let Ok(key) = crate::keyring_store::get_key(KEYRING_SERVICE, &account) {
 if !key.trim().is_empty() {
 return Ok(true);
 }
 }
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 Ok(db::get_provider_legacy_api_key(&conn, provider_id)
 .map_err(|e| e.to_string())?
 .is_some_and(|key| !key.trim().is_empty()))
}

fn hydrate_provider(
 state: &State<AppState>,
 mut provider: db::Provider,
) -> Result<db::Provider, String> {
 provider.has_api_key = provider_has_key(state, &provider.id)?;
 Ok(provider)
}

// ===== Provider commands =====

#[tauri::command]
pub fn list_providers(state: State<AppState>) -> Result<Vec<db::Provider>, String> {
 let providers = {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 db::list_providers(&conn).map_err(|e| e.to_string())?
 };
 providers
 .into_iter()
 .map(|provider| hydrate_provider(&state, provider))
 .collect()
}

#[tauri::command]
pub fn create_provider(
 state: State<AppState>,
 input: CreateProviderInput,
) -> Result<db::Provider, String> {
 let CreateProviderInput {
 id,
 name,
 api_base,
 anthropic_mode,
 api_key,
 } = input;

 // =====阶段1：确定最终 id（无副作用）=====
 let final_id = {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 let exists: bool = conn
 .prepare("SELECT1 FROM providers WHERE id = ?1")
 .map_err(|e| e.to_string())?
 .exists(rusqlite::params![id])
 .map_err(|e| e.to_string())?;
 if exists {
 let uuid_str = uuid::Uuid::new_v4().to_string();
 let suffix = uuid_str.split('-').next().unwrap_or("x");
 format!("{}-{}", id, suffix)
 } else {
 id.clone()
 }
 };

 let p = db::Provider {
 id: final_id.clone(),
 name,
 api_base,
 anthropic_mode,
 has_api_key: false,
 created_at: now_ts(),
 updated_at: now_ts(),
 };

 // =====阶段2：DB 先 commit =====
 let mut conn = state.db.lock().map_err(|e| e.to_string())?;
 keyring_db::run_tx(&mut conn, |tx| db::insert_provider(tx, &p))
 .map_err(|e| format!("写入失败: {}", e))?;
 drop(conn);

 // =====阶段3：Keyring 后写（DB 已 commit，Keyring失败仅 warn 不回滚 DB）=====
 if let Some(key) = api_key.as_deref().filter(|k| !k.is_empty()) {
 let account = provider_account(&p.id);
 if let Err(e) = keyring_db::write_keyring_one(KEYRING_SERVICE, &account, key) {
 eprintln!(
 "Warning:厂商 {} Keyring写入失败,DB 已保留: {}",
 p.id, e
 );
 return Err(e);
 }
 }
 hydrate_provider(&state, p)
}

#[tauri::command]
pub fn delete_provider(state: State<AppState>, id: String) -> Result<(), String> {
 // =====阶段1：收集要清理的 Keyring accounts（读，无副作用）=====
 let provider_key = provider_account(&id);
 let binding_accounts: Vec<String> = {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 db::list_bindings_by_provider(&conn, &id)
 .map_err(|e| e.to_string())?
 .into_iter()
 .filter_map(|b| b.keyring_key)
 .collect()
 };

 // =====阶段2：DB 先 commit（级联删除 bindings + models + provider）=====
 let mut conn = state.db.lock().map_err(|e| e.to_string())?;
 db::delete_provider_cascade(&mut *conn, &id).map_err(|e| e.to_string())?;
 drop(conn);

 // =====阶段3：Keyring best-effort清理（失败仅 warn，DB 已删除不应回滚）=====
 let mut all_accounts = binding_accounts;
 all_accounts.push(provider_key);
 keyring_db::delete_keyring_best_effort(KEYRING_SERVICE, &all_accounts);

 Ok(())
}

#[tauri::command]
pub fn update_provider(
 state: State<AppState>,
 input: CreateProviderInput,
) -> Result<db::Provider, String> {
 let CreateProviderInput {
 id,
 name,
 api_base,
 anthropic_mode,
 api_key,
 } = input;

 // =====阶段1：读 +改字段（无副作用）=====
 let mut conn = state.db.lock().map_err(|e| e.to_string())?;
 let mut provider = db::get_provider(&conn, &id)
 .map_err(|e| e.to_string())?
 .ok_or_else(|| "厂商未找到".to_string())?;
 provider.name = name;
 provider.api_base = api_base;
 provider.anthropic_mode = anthropic_mode;
 provider.updated_at = now_ts();

 // =====阶段2：DB事务内 update → commit（持锁时间缩短）=====
 keyring_db::run_tx(&mut conn, |tx| db::update_provider(tx, &provider))
 .map_err(|e| e.to_string())?;
 drop(conn);

 // =====阶段3：Keyring 后写（不在 DB锁内）=====
 if let Some(key) = api_key.as_deref().filter(|k| !k.is_empty()) {
 let account = provider_account(&provider.id);
 if let Err(e) = keyring_db::write_keyring_one(KEYRING_SERVICE, &account, key) {
 eprintln!(
 "Warning:厂商 {} Keyring写入失败,DB 已更新: {}",
 provider.id, e
 );
 return Err(e);
 }
 }
 hydrate_provider(&state, provider)
}

// ===== Model commands =====

#[derive(serde::Deserialize)]
pub struct CreateModelInput {
 pub provider_id: String,
 pub name: String,
 pub model_id: String,
 pub context_length: i64,
 pub max_output: i64,
}

#[derive(serde::Deserialize)]
pub struct UpdateModelInput {
 pub id: String,
 pub provider_id: String,
 pub name: String,
 pub model_id: String,
 pub context_length: i64,
 pub max_output: i64,
}

#[tauri::command]
pub fn list_models(state: State<AppState>) -> Result<Vec<db::Model>, String> {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 db::list_models(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_model(
 state: State<AppState>,
 input: CreateModelInput,
) -> Result<db::Model, String> {
 let id = format!("{}/{}", input.provider_id, input.model_id);
 let now = now_ts();
 let m = db::Model {
 id,
 provider_id: input.provider_id,
 name: input.name,
 model_id: input.model_id,
 context_length: input.context_length,
 max_output: input.max_output,
 supports_attachment: false,
 supports_reasoning: true,
 supports_tool_call: true,
 supports_vision: false,
 created_at: now,
 updated_at: now,
 };
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 db::insert_model(&conn, &m).map_err(|e| e.to_string())?;
 Ok(m)
}

#[tauri::command]
pub fn update_model(
 state: State<AppState>,
 input: UpdateModelInput,
) -> Result<db::Model, String> {
 let now = now_ts();
 let m = db::Model {
 id: input.id,
 provider_id: input.provider_id,
 name: input.name,
 model_id: input.model_id,
 context_length: input.context_length,
 max_output: input.max_output,
 supports_attachment: false,
 supports_reasoning: true,
 supports_tool_call: true,
 supports_vision: false,
 created_at: now,
 updated_at: now,
 };
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 db::update_model(&conn, &m).map_err(|e| e.to_string())?;
 Ok(m)
}

#[tauri::command]
pub fn delete_model(state: State<AppState>, id: String) -> Result<(), String> {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 db::delete_model(&conn, &id).map_err(|e| e.to_string())
}

// ===== Binding commands =====

#[tauri::command]
pub fn apply_binding(
 state: State<AppState>,
 tool_id: String,
 provider_id: String,
 model_id: String,
) -> Result<(), String> {
 let _tool_lock = get_tool_lock(&tool_id);
 let _tool_guard = _tool_lock.lock().map_err(|e| e.to_string())?;

 // =====阶段1：收集所有需要的信息（纯读，无副作用）=====
 let (model_name, api_key) = {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 let models =
 db::list_models_by_provider(&conn, &provider_id).map_err(|e| e.to_string())?;
 let model_name = models
 .into_iter()
 .find(|m| m.id == model_id)
 .map(|m| m.model_id)
 .ok_or_else(|| "模型未找到".to_string())?;
 drop(conn);
 let api_key = fetch_provider_key(&state, &provider_id)?;
 (model_name, api_key)
 };

 let binding_id = uuid::Uuid::new_v4().to_string();
 let keyring_key = binding_account(&binding_id);
 let now = now_ts();

 // =====阶段2：DB事务替换绑定（先 commit，后 config/keyring）=====
 //捕获旧绑定完整数据，用于必要时回滚（重新插入）。
 let old_bindings: Vec<db::ToolBinding> = {
 let mut conn = state.db.lock().map_err(|e| e.to_string())?;
 let old =
 db::list_bindings_by_tool(&conn, &tool_id).map_err(|e| e.to_string())?;

 keyring_db::run_tx(&mut conn, |tx| {
 tx.execute("DELETE FROM tool_bindings WHERE tool_id = ?1", [&tool_id])?;
 tx.execute(
 "INSERT INTO tool_bindings (id, tool_id, provider_id, model_id, keyring_key, is_active, created_at, updated_at)
 VALUES (?1, ?2, ?3, ?4, ?5,1, ?6, ?7)",
 rusqlite::params![binding_id, tool_id, provider_id, model_id, keyring_key, now, now],
 )?;
 Ok(())
 })?;

 old
 };

 // =====阶段3：写配置文件（atomic_write + backup 由 config_writer 保证）=====
 //失败时：回滚 DB（恢复旧绑定） +清理可能已写入的 keyring
 if let Err(e) = apply_config_for_tool(&state, &tool_id, &provider_id, &model_name, &api_key) {
 rollback_binding_db(&state, &tool_id, &old_bindings);
 return Err(e);
 }

 // =====阶段4：写 binding keyring（DB 已 commit，失败仅 warn）=====
 if let Err(e) = keyring_db::write_keyring_one(KEYRING_SERVICE, &keyring_key, &api_key) {
 eprintln!(
 "Warning:绑定 {} Keyring写入失败,DB 已更新: {}",
 binding_id, e
 );
 return Err(format!("Keyring写入失败: {}", e));
 }

 // =====阶段5：清理旧 binding keyrings（best-effort）=====
 let old_keys: Vec<String> = old_bindings
 .into_iter()
 .filter_map(|b| b.keyring_key)
 .collect();
 keyring_db::delete_keyring_best_effort(KEYRING_SERVICE, &old_keys);

 Ok(())
}

/// 回滚 DB绑定变更（apply_binding失败时调用）。
///
///重新插入旧绑定（如果有），删除新绑定。
fn rollback_binding_db(
 state: &State<AppState>,
 tool_id: &str,
 old_bindings: &[db::ToolBinding],
) {
 let mut conn = match state.db.lock() {
 Ok(c) => c,
 Err(e) => {
 eprintln!("Warning: 回滚绑定时获取 DB锁失败: {}", e);
 return;
 }
 };

 if let Err(e) = keyring_db::run_tx(&mut conn, |tx| {
 // 删除当前 tool 的所有绑定
 tx.execute("DELETE FROM tool_bindings WHERE tool_id = ?1", [tool_id])?;
 //恢复旧绑定
 for old in old_bindings {
 tx.execute(
 "INSERT OR REPLACE INTO tool_bindings
 (id, tool_id, provider_id, model_id, keyring_key, is_active, created_at, updated_at)
 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
 rusqlite::params![
 old.id, old.tool_id, old.provider_id, old.model_id,
 old.keyring_key, old.is_active as i32,
 old.created_at, now_ts()
 ],
 )?;
 }
 Ok(())
 }) {
 eprintln!("Warning: 回滚绑定 DB变更失败: {}", e);
 return;
 }

 //清理当前失败过程中可能已经写入的新 keyring（如果新 binding 已经写了 key）
 let key = binding_account(tool_id);
 keyring_db::delete_keyring_best_effort(KEYRING_SERVICE, &[key]);
}

#[tauri::command]
pub fn get_tool_binding(
 state: State<AppState>,
 tool_id: String,
) -> Result<Option<serde_json::Value>, String> {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 let binding = db::get_active_binding(&conn, &tool_id).map_err(|e| e.to_string())?;
 let Some(b) = binding else {
 return Ok(None);
 };
 let provider = db::get_provider(&conn, &b.provider_id).map_err(|e| e.to_string())?;
 let model = db::list_models(&conn)
 .map_err(|e| e.to_string())?
 .into_iter()
 .find(|m| m.id == b.model_id);
 Ok(Some(serde_json::json!({
 "id": b.id,
 "tool_id": b.tool_id,
 "provider_id": b.provider_id,
 "provider_name": provider.as_ref().map(|p| p.name.as_str()),
 "model_id": b.model_id,
 "model_name": model.as_ref().map(|m| m.name.as_str()),
 "is_active": b.is_active,
 })))
}

#[tauri::command]
pub fn unbind_tool(state: State<AppState>, binding_id: String) -> Result<(), String> {
 // =====阶段1：读 binding（无副作用）=====
 let binding = {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 db::list_bindings(&conn)
 .map_err(|e| e.to_string())?
 .into_iter()
 .find(|b| b.id == binding_id)
 };

 let Some(binding) = binding else {
 return Ok(());
 };

 // =====阶段2：DB 先 commit 删除 =====
 {
 let conn = state.db.lock().map_err(|e| e.to_string())?;
 db::delete_binding(&conn, &binding_id).map_err(|e| e.to_string())?;
 }

 // =====阶段3：Keyring best-effort清理 =====
 if let Some(account) = binding.keyring_key {
 keyring_db::delete_keyring_best_effort(KEYRING_SERVICE, &[account]);
 }

 Ok(())
}
