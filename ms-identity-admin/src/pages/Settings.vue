<template>
  <div>
    <h1 class="text-2xl font-bold mb-4">系统设置</h1>
    
    <!-- 缓存配置 -->
    <n-card class="mb-4">
      <template #header>
        <div class="flex items-center gap-2">
          <n-icon size="20" :component="SettingsOutline" />
          <span>缓存配置</span>
        </div>
      </template>
      <div class="flex items-center justify-between">
        <div>
          <div class="font-semibold">权限缓存开关</div>
          <div class="text-sm text-gray-500 mt-1">
            开启后，菜单和权限数据会缓存到本地，刷新页面不会重新请求。关闭后每次都会请求最新数据。
          </div>
        </div>
        <n-switch v-model:value="useCache" @update:value="handleCacheToggle" />
      </div>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NCard, NSwitch, useMessage, NIcon } from 'naive-ui';
import { SettingsOutline } from '@vicons/ionicons5';
import { usePermissionStore } from '@/store/modules/permission';

const message = useMessage();
const permissionStore = usePermissionStore();

const useCache = computed({
  get: () => permissionStore.useCache,
  set: (value: boolean) => permissionStore.setUseCache(value)
});

const handleCacheToggle = (value: boolean) => {
  permissionStore.setUseCache(value);
  message.success(value ? '已开启权限缓存' : '已关闭权限缓存，下次请求将获取最新数据');
};
</script>
