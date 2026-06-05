import { useEffect, useState } from 'react';
import { api, VendorInstance } from '../api';
import { Button } from './ui/button';
import { Card } from './ui/card';

export function VendorList({
  onAdd,
  onEdit,
  onChanged,
  refreshKey,
}: {
  onAdd: () => void;
  onEdit: (v: VendorInstance) => void;
  onChanged?: () => void;
  refreshKey: number;
}) {
  const [vendors, setVendors] = useState<VendorInstance[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = async () => {
    setLoading(true);
    try {
      const [list, active] = await Promise.all([api.listVendors(), api.getActiveVendor()]);
      setVendors(list);
      setActiveId(active);
    } finally {
      setLoading(false);
    }
  };

  // 跟随父组件传入的 refreshKey 变化触发重新拉取，
  // 这样 dialog 保存/删除/应用成功后父组件自增 listKey 即可通知子组件刷新。
  useEffect(() => {
    refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refreshKey]);

  const apply = async (id: string) => {
    try {
      await api.applyVendor(id);
      await refresh();
      onChanged?.();
    } catch (e) {
      alert('切换失败: ' + e);
    }
  };

  const remove = async (id: string) => {
    if (!confirm('确定删除此厂商？API Key 将从 Keyring 清除。')) return;
    try {
      await api.deleteVendor(id);
      await refresh();
      onChanged?.();
    } catch (e) {
      alert('删除失败: ' + e);
    }
  };

  return (
    <div>
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-lg font-semibold">厂商列表</h2>
        <Button onClick={onAdd}>+ 添加厂商</Button>
      </div>
      {loading && <p className="text-sm text-gray-500">加载中...</p>}
      {!loading && vendors.length === 0 && (
        <p className="text-sm text-gray-500">还没有厂商，点击右上角"添加厂商"开始。</p>
      )}
      <div className="space-y-2">
        {vendors.map((v) => (
          <Card key={v.id} className="p-4 flex justify-between items-center">
            <div>
              <div className="flex items-center gap-2">
                {v.id === activeId ? (
                  <span className="inline-block w-2 h-2 rounded-full bg-green-500" />
                ) : (
                  <span className="inline-block w-2 h-2 rounded-full bg-gray-300" />
                )}
                <span className="font-medium">{v.name}</span>
              </div>
              <div className="text-xs text-gray-500 mt-1">
                {v.api_base} · 模型: {v.model}
              </div>
            </div>
            <div className="flex gap-2">
              {v.id !== activeId && (
                <Button size="sm" onClick={() => apply(v.id)}>应用</Button>
              )}
              <Button size="sm" variant="outline" onClick={() => onEdit(v)}>编辑</Button>
              <Button size="sm" variant="destructive" onClick={() => remove(v.id)}>删除</Button>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}
