import { useQuery } from '@tanstack/react-query';
import { api } from '@/api';
import { useAppMutation } from './useAppMutation';

export function useModels() {
  return useQuery({
    queryKey: ['models'],
    queryFn: () => api.listModels(),
  });
}

export function useCreateModel() {
  return useAppMutation({
    mutationFn: (input: Parameters<typeof api.createModel>[0]) =>
      api.createModel(input),
    successMsg: '模型添加成功',
    invalidateKeys: [['models']],
  });
}

export function useUpdateModel() {
  return useAppMutation({
    mutationFn: (input: Parameters<typeof api.updateModel>[0]) =>
      api.updateModel(input),
    successMsg: '模型更新成功',
    invalidateKeys: [['models']],
  });
}

export function useDeleteModel() {
  return useAppMutation({
    mutationFn: (id: string) => api.deleteModel(id),
    successMsg: '模型已删除',
    invalidateKeys: [['models']],
  });
}
