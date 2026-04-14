<template>
  <div class="dashboard-container">
    <!-- 系统概览与统计数据 -->
    <n-card class="mb-6">
      <template #header>
        <div class="flex items-center gap-2">
          <n-icon :component="BarChartOutline" :size="20" />
          <span class="font-semibold">系统概览与统计数据</span>
        </div>
      </template>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      <!-- 用户总数卡片 -->
      <n-card class="stat-card stat-card-blue" hoverable>
        <div class="flex items-center justify-between">
          <div class="flex-1">
            <div class="text-sm text-gray-500 dark:text-gray-400 mb-2">用户总数</div>
            <div class="text-4xl font-bold text-gray-800 dark:text-gray-100 mb-1">
              {{ loading ? '--' : formatNumber(userCount) }}
            </div>
            <div class="text-xs text-gray-400 dark:text-gray-500 mt-1">所有注册用户</div>
          </div>
          <div class="stat-icon stat-icon-blue">
            <n-icon :component="PeopleOutline" :size="48" />
          </div>
        </div>
      </n-card>
    
      <!-- 租户总数卡片 -->
      <n-card class="stat-card stat-card-green" hoverable>
        <div class="flex items-center justify-between">
          <div class="flex-1">
            <div class="text-sm text-gray-500 dark:text-gray-400 mb-2">租户总数</div>
            <div class="text-4xl font-bold text-gray-800 dark:text-gray-100 mb-1">
              {{ loading ? '--' : formatNumber(tenantCount) }}
            </div>
            <div class="text-xs text-gray-400 dark:text-gray-500 mt-1">所有租户组织</div>
          </div>
          <div class="stat-icon stat-icon-green">
            <n-icon :component="BusinessOutline" :size="48" />
          </div>
        </div>
      </n-card>

      <!-- 活跃用户卡片 -->
      <n-card class="stat-card stat-card-purple" hoverable>
      <div class="flex items-center justify-between">
          <div class="flex-1">
            <div class="text-sm text-gray-500 dark:text-gray-400 mb-2">活跃用户</div>
            <div class="text-4xl font-bold text-gray-800 dark:text-gray-100 mb-1">
              {{ loading ? '--' : formatNumber(activeUserCount) }}
            </div>
            <div class="text-xs text-gray-400 dark:text-gray-500 mt-1">最近30天内有登录</div>
          </div>
          <div class="stat-icon stat-icon-purple">
            <n-icon :component="PulseOutline" :size="48" />
          </div>
        </div>
      </n-card>
      </div>
    </n-card>

    <!-- 数据概览卡片 -->
    <n-card class="mb-6">
      <template #header>
        <div class="flex items-center gap-2">
          <n-icon :component="BarChartOutline" :size="20" />
          <span class="font-semibold">数据概览</span>
        </div>
      </template>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div class="overview-item">
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm text-gray-600 dark:text-gray-300">活跃率</span>
            <span class="text-sm font-semibold text-gray-800 dark:text-gray-100">
              {{ loading ? '--' : calculateActiveRate() }}%
            </span>
          </div>
          <n-progress
            :percentage="loading ? 0 : calculateActiveRate()"
            :show-indicator="false"
            :height="8"
            status="success"
          />
        </div>
        <div class="overview-item">
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm text-gray-600 dark:text-gray-300">平均每租户用户数</span>
            <span class="text-sm font-semibold text-gray-800 dark:text-gray-100">
              {{ loading ? '--' : calculateAvgUsersPerTenant() }}
            </span>
          </div>
          <n-progress
            :percentage="loading ? 0 : Math.min(calculateAvgUsersPerTenant() * 10, 100)"
            :show-indicator="false"
            :height="8"
            status="info"
          />
        </div>
        </div>
      </n-card>

    <!-- 快速操作 -->
      <n-card>
      <template #header>
        <div class="flex items-center gap-2">
          <n-icon :component="FlashOutline" :size="20" />
          <span class="font-semibold">快速操作</span>
        </div>
      </template>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
        <n-button
          type="primary"
          ghost
          class="quick-action-btn"
          @click="$router.push({ name: 'Users' })"
        >
          <template #icon>
            <n-icon :component="PeopleOutline" />
          </template>
          用户管理
        </n-button>
        <n-button
          type="info"
          ghost
          class="quick-action-btn"
          @click="$router.push({ name: 'Tenants' })"
        >
          <template #icon>
            <n-icon :component="BusinessOutline" />
          </template>
          租户管理
        </n-button>
        <n-button
          type="success"
          ghost
          class="quick-action-btn"
          @click="$router.push({ name: 'Roles' })"
        >
          <template #icon>
            <n-icon :component="ShieldCheckmarkOutline" />
          </template>
          角色管理
        </n-button>
        <n-button
          type="warning"
          ghost
          class="quick-action-btn"
          @click="$router.push({ name: 'Settings' })"
        >
          <template #icon>
            <n-icon :component="SettingsOutline" />
          </template>
          系统设置
        </n-button>
        </div>
      </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
  NCard,
  NIcon,
  NProgress,
  NButton,
  useMessage,
  useLoadingBar
} from 'naive-ui';
import {
  PeopleOutline,
  BusinessOutline,
  PulseOutline,
  BarChartOutline,
  FlashOutline,
  ShieldCheckmarkOutline,
  SettingsOutline
} from '@vicons/ionicons5';
import { userApi } from '@/api/user';
import { tenantApi } from '@/api/tenant';
import { handleApiResult } from '@/utils/request';

const message = useMessage();
const loadingBar = useLoadingBar();

const loading = ref(false);
const userCount = ref<number>(0);
const tenantCount = ref<number>(0);
const activeUserCount = ref<number>(0);

// 格式化数字（添加千分位）
const formatNumber = (num: number): string => {
  return num.toLocaleString('zh-CN');
};

// 计算活跃率
const calculateActiveRate = (): number => {
  if (userCount.value === 0) return 0;
  return Math.round((activeUserCount.value / userCount.value) * 100);
};

// 计算平均每租户用户数
const calculateAvgUsersPerTenant = (): number => {
  if (tenantCount.value === 0) return 0;
  return Math.round((userCount.value / tenantCount.value) * 10) / 10;
};

const loadStatistics = async () => {
  loading.value = true;
  loadingBar.start();
  try {
    // 并行加载所有统计数据
    const [userResponse, tenantResponse, activeUserResponse] = await Promise.all([
      userApi.getUserCount(),
      tenantApi.getTenantCount(),
      userApi.getActiveUserCount(30)
    ]);

    const userCountData = handleApiResult(userResponse);
    if (userCountData !== null && typeof userCountData === 'number') {
      userCount.value = userCountData;
    }

    const tenantCountData = handleApiResult(tenantResponse);
    if (tenantCountData !== null && typeof tenantCountData === 'number') {
      tenantCount.value = tenantCountData;
    }

    const activeUserCountData = handleApiResult(activeUserResponse);
    if (activeUserCountData !== null && typeof activeUserCountData === 'number') {
      activeUserCount.value = activeUserCountData;
    }
  } catch (error: any) {
    message.error(error.message || '加载统计数据失败');
  } finally {
    loading.value = false;
    loadingBar.finish();
  }
};

onMounted(() => {
  loadStatistics();
});
</script>

<style scoped>
.dashboard-container {
  max-width: 1400px;
  margin: 0 auto;
}

.stat-card {
  border-radius: 12px;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}

.stat-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
  background: linear-gradient(90deg, transparent, currentColor, transparent);
  opacity: 0.3;
}

.stat-card-blue::before {
  color: #3b82f6;
}

.stat-card-green::before {
  color: #10b981;
}

.stat-card-purple::before {
  color: #8b5cf6;
}

.stat-icon {
  width: 80px;
  height: 80px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.1;
  transition: opacity 0.3s ease;
}

.stat-card:hover .stat-icon {
  opacity: 0.2;
}

.stat-icon-blue {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  color: #3b82f6;
}

.stat-icon-green {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: #10b981;
}

.stat-icon-purple {
  background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
  color: #8b5cf6;
}

.overview-item {
  padding: 16px;
  background: rgba(0, 0, 0, 0.02);
  border-radius: 8px;
}

.dark .overview-item {
  background: rgba(255, 255, 255, 0.05);
}

.quick-action-btn {
  height: 80px;
  flex-direction: column;
  gap: 8px;
}

.quick-action-btn :deep(.n-button__icon) {
  font-size: 24px;
}
</style>
