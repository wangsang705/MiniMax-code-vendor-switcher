import {
  useMutation,
  useQueryClient,
  type QueryKey,
} from '@tanstack/react-query';
import { useToast } from './use-toast';

interface UseAppMutationOptions<TVariables, TData> {
  mutationFn: (variables: TVariables) => Promise<TData>;
  successMsg?: string;
  errorMsg?: string;
  invalidateKeys?: QueryKey[];
  onSuccess?: (data: TData, variables: TVariables) => void;
}

export function useAppMutation<TVariables, TData = void>(
  options: UseAppMutationOptions<TVariables, TData>,
) {
  const toast = useToast();
  const queryClient = useQueryClient();

  return useMutation<TData, Error, TVariables>({
    mutationFn: options.mutationFn,
    onSuccess: (data, variables) => {
      if (options.successMsg) {
        toast.success(options.successMsg);
      }
      options.invalidateKeys?.forEach((key) => {
        queryClient.invalidateQueries({ queryKey: key });
      });
      options.onSuccess?.(data, variables);
    },
    onError: (error) => {
      toast.error(options.errorMsg ?? error.message ?? '操作失败');
    },
  });
}
