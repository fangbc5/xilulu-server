import { defineStore } from 'pinia';
import { ref } from 'vue';
import { userApi } from '@/api/user';
import { printApiError } from '@/utils/request';
import { usePermissionStore } from './permission';

export interface IUserState {
  id: number;
  username?: string;
  email?: string;
  mobile?: string;
  tenant_id?: number;
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('access_token'));
  const refreshToken = ref<string | null>(localStorage.getItem('refresh_token'));
  const user = ref<IUserState | null>(null);

  const setToken = (accessToken: string, refreshTokenValue: string) => {
    token.value = accessToken;
    refreshToken.value = refreshTokenValue;
    localStorage.setItem('access_token', accessToken);
    localStorage.setItem('refresh_token', refreshTokenValue);
  };

  const clearToken = () => {
    token.value = null;
    refreshToken.value = null;
    user.value = null;
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
    // 清理权限缓存
    const permissionStore = usePermissionStore();
    permissionStore.clear();
  };

  const login = async (loginInfo: any) => {
    const response = await userApi.login(loginInfo);
    const result = response.data;
    if (result && result.success && result.code === 200 && result.data) {
      // 后端 LoginResponse 返回 access_token 和 refresh_token
      const accessToken = result.data.access_token;
      const refreshTokenValue = result.data.refresh_token || accessToken;
      setToken(accessToken, refreshTokenValue);
      return true;
    }
    // 抛出后端返回的错误信息，由调用方显示
    throw new Error(result?.msg || '登录失败');
  };

  const logout = async () => {
    try {
      await userApi.logout();
    } catch (error: any) {
      printApiError(error);
    } finally {
      clearToken();
    }
  };

  return {
    token,
    refreshToken,
    user,
    setToken,
    clearToken,
    login,
    logout
  };
});

