import { useQuery } from '@tanstack/react-query';
import { api } from '@/api';
import { useAppMutation } from './useAppMutation';

export function useProviders() {
  return useQuery({
    queryKey: ['providers'],
    queryFn: () => api.listProviders(),
  });
}

export function useCreateProvider() {
  return useAppMutation({
    mutationFn: (input: Parameters<typeof api.createProvider>[0]) =>
      api.createProvider(input),
    successMsg: '厂商创建成功',
    invalidateKeys: [['providers']],
  });
}

export function useUpdateProvider() {
  return useAppMutation({
    mutationFn: (input: Parameters<typeof api.updateProvider>[0]) =>
      api.updateProvider(input),
    successMsg: '厂商更新成功',
    invalidateKeys: [['providers']],
  });
}

export function useDeleteProvider() {
  return useAppMutation({
    mutationFn: (id: string) => api.deleteProvider(id),
    successMsg: '厂商已删除',
    invalidateKeys: [['providers'], ['models']],
  });
}
