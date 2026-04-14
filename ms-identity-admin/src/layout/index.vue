<template>
  <n-layout class="h-screen">
    <n-layout-header bordered class="h-12 flex items-center px-4">
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center justify-center w-60 gap-2">
          <div class="flex items-center justify-center" style="width: 24px; height: 24px;">
            <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" style="width: 100%; height: 100%;">
              <path
                d="M12 2L2 7L12 12L22 7L12 2Z"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M2 17L12 22L22 17"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M2 12L12 17L22 12"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </div>
          <h1 class="text-base font-bold">运营管理后台</h1>
        </div>
        <div class="flex items-center gap-4">
          <n-button quaternary size="small" @click="handleLogout">登出</n-button>
        </div>
      </div>
    </n-layout-header>
    <n-layout has-sider class="h-[calc(100vh-3rem)]">
      <n-layout-sider
        bordered
        show-trigger
        collapse-mode="width"
        :collapsed-width="64"
        :width="240"
        :native-scrollbar="false"
      >
        <n-menu
          :options="menuOptions"
          :value="currentRoute"
          @update:value="handleMenuSelect"
          :collapsed-width="64"
          :collapsed-icon-size="22"
          :loading="menuLoading"
        />
      </n-layout-sider>
      <n-layout-content class="p-4 overflow-auto">
        <router-view />
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<script setup lang="ts">
import { computed, h, ref, onMounted, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { NLayout, NLayoutHeader, NLayoutSider, NLayoutContent, NMenu, NButton, NIcon } from 'naive-ui';
import { useAuthStore } from '@/store/modules/auth';
import { usePermissionStore } from '@/store/modules/permission';
import { resourceApi } from '@/api/resource';
import { handleApiResult } from '@/utils/request';
import { IResourceInfo } from '@/types/base';
import {
  HomeOutline,
  PeopleOutline,
  BusinessOutline,
  ShieldCheckmarkOutline,
  DocumentTextOutline,
  AppsOutline,
  CubeOutline,
  SettingsOutline
} from '@vicons/ionicons5';

// Icon映射表
const iconMap: Record<string, any> = {
  HomeOutline,
  PeopleOutline,
  BusinessOutline,
  ShieldCheckmarkOutline,
  DocumentTextOutline,
  AppsOutline,
  CubeOutline,
  SettingsOutline
};

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();
const permissionStore = usePermissionStore();

const currentRoute = computed(() => route.name as string);
const menuOptions = ref<any[]>([]);
const menuLoading = ref(false);

// 根据icon名称获取图标组件
const getIconComponent = (iconName?: string) => {
  if (!iconName) return null;
  const IconComponent = iconMap[iconName];
  if (!IconComponent) return null;
  return () => h(NIcon, { component: IconComponent });
};

// 加载用户菜单
const loadMenus = async () => {
  menuLoading.value = true;
  try {
    // 如果开启缓存且已有缓存菜单，直接使用缓存，不再请求
    if (permissionStore.useCache && permissionStore.menus && permissionStore.menus.length > 0) {
      menuOptions.value = permissionStore.menus
        .filter((resource: IResourceInfo) => resource.resource_type === '20')
        .map((resource: IResourceInfo) => ({
          label: resource.name,
          key: resource.component || resource.code,
          icon: getIconComponent(resource.icon),
          path: resource.path,
          menuId: resource.id
        }));
      return;
    }

    // TODO: 从配置或store中获取application_id和tenant_id
    const applicationId = 1; // 运营管理平台的应用ID，需要从配置或store获取
    const tenantId = authStore.user?.tenant_id;

    const response = await resourceApi.getCurrentUserMenus(applicationId, tenantId);
    const data = handleApiResult(response);
    
    if (data && Array.isArray(data)) {
      // 缓存菜单资源
      permissionStore.setMenus(data);
      // 将资源转换为菜单选项
      menuOptions.value = data
        .filter((resource: IResourceInfo) => resource.resource_type === '20') // 只显示菜单类型
        .map((resource: IResourceInfo) => ({
          label: resource.name,
          key: resource.component || resource.code, // 使用component作为路由name，如果没有则使用code
          icon: getIconComponent(resource.icon),
          path: resource.path,
          menuId: resource.id
        }));
    }
  } catch (error: any) {
    console.error('加载菜单失败:', error);
    // 如果加载失败，使用默认菜单
    menuOptions.value = [
      {
        label: '仪表盘',
        key: 'Dashboard',
        icon: () => h(NIcon, { component: HomeOutline })
      }
    ];
  } finally {
    menuLoading.value = false;
  }
};

const handleMenuSelect = (key: string) => {
  const option = menuOptions.value.find(item => item.key === key) as any;
  if (option && option.menuId) {
    permissionStore.setCurrentMenuId(option.menuId as number);
  } else {
    permissionStore.setCurrentMenuId(null);
  }
  router.push({ name: key });
};

const handleLogout = async () => {
  await authStore.logout();
  router.push('/login');
};

// 根据当前路由恢复菜单ID（刷新页面后使用）
const restoreCurrentMenuId = () => {
  const routeName = route.name as string;
  if (routeName) {
    const option = menuOptions.value.find(item => item.key === routeName) as any;
    if (option && option.menuId) {
      permissionStore.setCurrentMenuId(option.menuId as number);
    }
  }
};

onMounted(async () => {
  await loadMenus();
  // 菜单加载完成后，根据当前路由恢复菜单ID
  restoreCurrentMenuId();
});

// 监听路由变化，更新菜单选中状态
watch(
  () => route.name,
  () => {
    restoreCurrentMenuId();
  }
);
</script>

<style scoped>
/* 收起状态下菜单项内容居中 */
:deep(.n-menu-item-content--collapsed) {
  justify-content: center;
}

/* 收起状态下图标居中 */
:deep(.n-menu-item-content--collapsed .n-icon) {
  margin: 0 !important;
}
</style>
