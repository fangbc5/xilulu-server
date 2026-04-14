import { defineStore } from 'pinia';
import { ref } from 'vue';
import { resourceApi } from '@/api/resource';
import { handleApiResult } from '@/utils/request';
import type { IMenuResourcesByType, IResourceInfo } from '@/types/base';

const MENUS_CACHE_KEY = 'permission_menus_cache';
const MENU_RESOURCES_CACHE_KEY = 'permission_menu_resources_cache';
const MENU_CODES_CACHE_KEY = 'permission_menu_codes_cache';
const USE_CACHE_KEY = 'permission_use_cache';

// 从 localStorage 读取缓存开关配置（默认开启缓存）
const loadUseCacheSetting = (): boolean => {
  try {
    const cached = localStorage.getItem(USE_CACHE_KEY);
    if (cached !== null) {
      return JSON.parse(cached);
    }
  } catch (e) {
    console.warn('读取缓存开关配置失败:', e);
  }
  return true; // 默认开启缓存
};

// 从 sessionStorage 读取菜单缓存
const loadMenusFromCache = (): IResourceInfo[] | null => {
  try {
    const cached = sessionStorage.getItem(MENUS_CACHE_KEY);
    if (cached) {
      return JSON.parse(cached);
    }
  } catch (e) {
    console.warn('读取菜单缓存失败:', e);
  }
  return null;
};

// 保存菜单到 sessionStorage
const saveMenusToCache = (menus: IResourceInfo[]) => {
  try {
    sessionStorage.setItem(MENUS_CACHE_KEY, JSON.stringify(menus));
  } catch (e) {
    console.warn('保存菜单缓存失败:', e);
  }
};

// 从 sessionStorage 读取菜单资源缓存
const loadMenuResourcesFromCache = (): Record<number, IMenuResourcesByType> => {
  try {
    const cached = sessionStorage.getItem(MENU_RESOURCES_CACHE_KEY);
    if (cached) {
      return JSON.parse(cached);
    }
  } catch (e) {
    console.warn('读取菜单资源缓存失败:', e);
  }
  return {};
};

// 保存菜单资源到 sessionStorage
const saveMenuResourcesToCache = (resources: Record<number, IMenuResourcesByType>) => {
  try {
    sessionStorage.setItem(MENU_RESOURCES_CACHE_KEY, JSON.stringify(resources));
  } catch (e) {
    console.warn('保存菜单资源缓存失败:', e);
  }
};

// 从 sessionStorage 读取菜单权限 code 缓存（Set 需要特殊处理）
const loadMenuCodesFromCache = (): Record<number, Set<string>> => {
  try {
    const cached = sessionStorage.getItem(MENU_CODES_CACHE_KEY);
    if (cached) {
      const parsed = JSON.parse(cached);
      const result: Record<number, Set<string>> = {};
      for (const [key, value] of Object.entries(parsed)) {
        result[Number(key)] = new Set(value as string[]);
      }
      return result;
    }
  } catch (e) {
    console.warn('读取菜单权限缓存失败:', e);
  }
  return {};
};

// 保存菜单权限 code 到 sessionStorage（Set 转为数组）
const saveMenuCodesToCache = (codes: Record<number, Set<string>>) => {
  try {
    const serializable: Record<string, string[]> = {};
    for (const [key, value] of Object.entries(codes)) {
      serializable[key] = Array.from(value);
    }
    sessionStorage.setItem(MENU_CODES_CACHE_KEY, JSON.stringify(serializable));
  } catch (e) {
    console.warn('保存菜单权限缓存失败:', e);
  }
};

export const usePermissionStore = defineStore('permission', () => {
  // 是否使用缓存（从 localStorage 初始化，默认 true）
  const useCache = ref<boolean>(loadUseCacheSetting());
  // 菜单列表缓存（从 sessionStorage 初始化）
  const menus = ref<IResourceInfo[] | null>(useCache.value ? loadMenusFromCache() : null);
  // 按菜单ID缓存子资源（从 sessionStorage 初始化）
  const menuResources = ref<Record<number, IMenuResourcesByType>>(
    useCache.value ? loadMenuResourcesFromCache() : {}
  );
  // 菜单权限 code 集合（从 sessionStorage 初始化）
  const menuCodes = ref<Record<number, Set<string>>>(
    useCache.value ? loadMenuCodesFromCache() : {}
  );
  const currentMenuId = ref<number | null>(null);

  const setCurrentMenuId = (menuId: number | null) => {
    currentMenuId.value = menuId;
  };

  const setMenus = (list: IResourceInfo[]) => {
    menus.value = list;
    if (useCache.value) {
      saveMenusToCache(list); // 同时保存到 sessionStorage
    }
  };

  // 设置是否使用缓存
  const setUseCache = (value: boolean) => {
    useCache.value = value;
    try {
      localStorage.setItem(USE_CACHE_KEY, JSON.stringify(value));
    } catch (e) {
      console.warn('保存缓存开关配置失败:', e);
    }
    // 如果关闭缓存，清除现有缓存
    if (!value) {
      menus.value = null;
      menuResources.value = {};
      menuCodes.value = {};
      sessionStorage.removeItem(MENUS_CACHE_KEY);
      sessionStorage.removeItem(MENU_RESOURCES_CACHE_KEY);
      sessionStorage.removeItem(MENU_CODES_CACHE_KEY);
    }
  };

  const buildCodes = (menuId: number, resources: IMenuResourcesByType) => {
    const codes = new Set<string>();
    (resources.menus || []).forEach(r => codes.add(r.code));
    (resources.buttons || []).forEach(r => codes.add(r.code));
    (resources.fields || []).forEach(r => codes.add(r.code));
    (resources.data || []).forEach(r => codes.add(r.code));
    menuCodes.value[menuId] = codes;
    if (useCache.value) {
      saveMenuCodesToCache(menuCodes.value); // 保存到 sessionStorage
    }
  };

  /**
   * 确保指定菜单ID的权限已加载（根据缓存开关决定是否使用缓存）
   */
  const ensureMenuResources = async (params: {
    application_id: number;
    menu_id: number;
    tenant_id?: number;
  }): Promise<IMenuResourcesByType | undefined> => {
    const { application_id, menu_id, tenant_id } = params;
    // 如果开启缓存且已有缓存，直接返回
    if (useCache.value && menuResources.value[menu_id]) {
      return menuResources.value[menu_id];
    }
    // 否则请求接口
    const resp = await resourceApi.getMenuResources({
      application_id,
      menu_id,
      tenant_id
    });
    const data = handleApiResult(resp) as IMenuResourcesByType | null;
    if (data) {
      menuResources.value[menu_id] = data;
      if (useCache.value) {
        saveMenuResourcesToCache(menuResources.value); // 保存到 sessionStorage
      }
      buildCodes(menu_id, data);
    }
    return menuResources.value[menu_id];
  };

  /**
   * 判断当前菜单（或指定菜单）的权限
   */
  const hasPermission = (code: string | string[], menuId?: number): boolean => {
    const id = menuId ?? currentMenuId.value;
    if (!id) return false;
    const set = menuCodes.value[id];
    if (!set) return false;
    const required = Array.isArray(code) ? code : [code];
    return required.some(c => set.has(c));
  };

  const clear = () => {
    menus.value = null;
    menuResources.value = {};
    menuCodes.value = {};
    currentMenuId.value = null;
    // 清除 sessionStorage 缓存
    sessionStorage.removeItem(MENUS_CACHE_KEY);
    sessionStorage.removeItem(MENU_RESOURCES_CACHE_KEY);
    sessionStorage.removeItem(MENU_CODES_CACHE_KEY);
  };

  return {
    menus,
    menuResources,
    menuCodes,
    currentMenuId,
    useCache,
    setCurrentMenuId,
    setMenus,
    setUseCache,
    ensureMenuResources,
    hasPermission,
    clear
  };
});


