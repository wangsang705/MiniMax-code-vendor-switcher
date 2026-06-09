import { useQuery } from '@tanstack/react-query';
import { api } from '@/api';
import { useAppMutation } from './useAppMutation';

export function useTools() {
  return useQuery({
    queryKey: ['tools'],
    queryFn: () => api.listTools(),
    retry: 2,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 10000),
    staleTime: 30_000,
  });
}

export function useDetectInstalledTools() {
  return useAppMutation({
    mutationFn: () => api.detectInstalledTools(),
    successMsg: '检测完成',
    invalidateKeys: [['tools']],
  });
}

export function useToolBinding(toolId: string | undefined) {
  return useQuery({
    queryKey: ['tool-binding', toolId],
    queryFn: () => api.getToolBinding(toolId!),
    enabled: !!toolId,
    retry: 1,
  });
}

export function useApplyBinding() {
  return useAppMutation({
    mutationFn: (params: {
      tool_id: string;
      provider_id: string;
      model_id: string;
    }) => api.applyBinding(params.tool_id, params.provider_id, params.model_id),
    successMsg: '绑定成功',
    invalidateKeys: [['tools'], ['tool-binding']],
  });
}

export function useUnbindTool() {
  return useAppMutation({
    mutationFn: (bindingId: string) => api.unbindTool(bindingId),
    successMsg: '已解除绑定',
    invalidateKeys: [['tools'], ['tool-binding']],
  });
}

export function useLaunchTool() {
  return useAppMutation({
    mutationFn: (toolId: string) => api.launchTool(toolId),
    successMsg: '工具已启动',
  });
}

export function useInstallTool() {
  return useAppMutation({
    mutationFn: (toolId: string) => api.installTool(toolId),
    successMsg: '工具安装成功',
    invalidateKeys: [['tools']],
  });
}
