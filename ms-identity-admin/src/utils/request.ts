import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import router from '@/router';
import { IApiResult } from '@/types/base';

/** Token 刷新冷却时间（ms），在此窗口内复用上次刷新结果 */
const REFRESH_COOLDOWN_MS = 5_000;

/** 是否正在刷新 */
let isRefreshing = false;
/** 等待刷新完成的请求队列 */
let refreshQueue: Array<{ resolve: (token: string) => void; reject: (err: Error) => void }> = [];
/** 最近一次刷新成功的 token */
let lastRefreshedToken: string | null = null;
/** 最近一次刷新成功的时间戳 */
let lastRefreshTime = 0;

// 刷新 token（并发安全 + 冷却窗口）
const doRefreshToken = async (): Promise<string> => {
  const refreshTokenStr = localStorage.getItem('refresh_token');
  if (!refreshTokenStr) {
    throw new Error('No refresh token');
  }

  // 冷却期内直接复用上次刷新的 token
  if (lastRefreshedToken && Date.now() - lastRefreshTime < REFRESH_COOLDOWN_MS) {
    return lastRefreshedToken;
  }

  // 并发请求进入等待队列
  if (isRefreshing) {
    return new Promise<string>((resolve, reject) => {
      refreshQueue.push({ resolve, reject });
    });
  }

  isRefreshing = true;

  try {
    const response = await axios.post('/api/v1/auth/refresh-token', {
      refresh_token: refreshTokenStr
    });

    if (response.data.success && response.data.code === 200) {
      const newToken = response.data.data.access_token;
      localStorage.setItem('access_token', newToken);
      localStorage.setItem('refresh_token', response.data.data.refresh_token);

      // 缓存刷新结果（用于冷却期内复用）
      lastRefreshedToken = newToken;
      lastRefreshTime = Date.now();

      // 通知等待队列
      refreshQueue.forEach((cb) => cb.resolve(newToken));

      return newToken;
    } else {
      throw new Error('Refresh token failed');
    }
  } catch (error) {
    // 通知等待队列
    refreshQueue.forEach((cb) => cb.reject(error as Error));
    throw error;
  } finally {
    isRefreshing = false;
    refreshQueue = [];
  }
};

class HttpRequest {
  constructor() {}

  interceptors(instance: AxiosInstance) {
    // 请求拦截器
    instance.interceptors.request.use((config) => {
      const token = localStorage.getItem('access_token');
      if (token) {
        config.headers['Authorization'] = `Bearer ${token}`;
      }
      return config;
    });

    // 响应拦截器
    instance.interceptors.response.use(
      (response) => {
        return response;
      },
      async (error) => {
        const originalRequest = error.config;

        // 401 错误（token 失效）且未重试过
        if (error.response?.status === 401 && !originalRequest._retry) {
          // 白名单内的接口返回 401 不需要刷新 token，直接抛出
          const noRefreshWhitelist = ['/auth/login', '/auth/register', '/auth/logout'];
          const url = originalRequest.url || '';
          if (noRefreshWhitelist.some(path => url.includes(path))) {
            const msg = error.response?.data?.msg || '认证失败';
            return Promise.reject(new Error(msg));
          }

          originalRequest._retry = true;

          try {
            const newToken = await doRefreshToken();
            originalRequest.headers['Authorization'] = `Bearer ${newToken}`;
            return instance(originalRequest);
          } catch (refreshError) {
            console.warn('[auth] refresh token 失败，清除 token 并跳转登录', refreshError);
            localStorage.removeItem('access_token');
            localStorage.removeItem('refresh_token');
            router.push(
              '/login?redirect=' +
              encodeURIComponent(location.pathname + location.search)
            );
            return Promise.reject(refreshError);
          }
        }

        return Promise.reject(error);
      }
    );
  }

  getInsideConfig(): AxiosRequestConfig {
    return {
      timeout: 15000,
    };
  }

  getJsonInsideConfig(): AxiosRequestConfig {
    return {
      timeout: 15000,
      headers: {
        'Content-Type': 'application/json'
      }
    };
  }

  request(options: AxiosRequestConfig): Promise<AxiosResponse> {
    const instance = axios.create();
    const config: AxiosRequestConfig = this.getInsideConfig();
    // GET请求不设置Content-Type
    if (options.method?.toLowerCase() !== 'get') {
      config.headers = {
        'Content-Type': 'application/x-www-form-urlencoded'
      };
    }
    options = Object.assign(config, options);
    this.interceptors(instance);
    return instance(options);
  }

  requestJSON(options: AxiosRequestConfig): Promise<AxiosResponse> {
    const instance = axios.create();
    const config: AxiosRequestConfig = this.getJsonInsideConfig();
    // GET请求不设置Content-Type
    if (options.method?.toLowerCase() === 'get') {
      if (config.headers && 'Content-Type' in config.headers) {
        const headers = config.headers as Record<string, any>;
        delete headers['Content-Type'];
      }
    }
    options = Object.assign(config, options);
    this.interceptors(instance);
    return instance(options);
  }
}

const request = new HttpRequest();
export default request;

export const handleApiResult = function <T>(
  response: AxiosResponse<IApiResult<T>>
): T | null {
  if (response.status === 200 && response.data.success && response.data.code === 200) {
    return response.data.data;
  } else {
    const errorMsg = response.data.msg || '请求失败';
    throw new Error(errorMsg);
  }
};

export const printApiSuccess = function (message?: string) {
  // 消息通过组件内的 useMessage() 显示，这里只做日志记录
  console.log(message || '操作成功');
};

export const printApiError = function (err: any) {
  // 错误通过组件内的 useMessage() 显示，这里只做日志记录
  console.error(err.message || '操作失败');
};
