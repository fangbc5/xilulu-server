<template>
  <div>
    <h1 class="text-2xl font-bold mb-4">资源管理</h1>
    <n-card>
      <template #header>
        <div class="flex justify-between items-center">
          <span>资源列表</span>
          <div class="flex gap-2">
            <n-select
              v-model:value="selectedTenantId"
              :options="tenantOptions"
              placeholder="选择租户"
              clearable
              filterable
              style="width: 200px"
              @update:value="handleTenantChange"
            />
            <n-select
              v-model:value="selectedApplicationId"
              :options="applicationOptions"
              placeholder="选择应用"
              clearable
              filterable
              style="width: 200px"
              :disabled="!selectedTenantId"
              @update:value="handleApplicationChange"
            />
            <n-input
              v-model:value="searchKey"
              placeholder="搜索资源代码或名称"
              clearable
              style="width: 300px"
              @keyup.enter="handleSearch"
            />
            <n-button @click="handleSearch">搜索</n-button>
            <n-button type="primary" @click="handleCreate">创建资源</n-button>
          </div>
        </div>
      </template>
      <n-data-table
        :columns="columns"
        :data="resources"
        :loading="loading"
        :pagination="pagination"
        remote
      />
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, computed, reactive } from 'vue';
import { useRouter } from 'vue-router';
import {
  NCard,
  NButton,
  NDataTable,
  DataTableColumns,
  useDialog,
  useMessage,
  NButton as NButtonComponent,
  NInput,
  NIcon,
  NTooltip,
  NTag,
  NSelect
} from 'naive-ui';
import { PencilOutline, TrashOutline } from '@vicons/ionicons5';
import { resourceApi } from '@/api/resource';
import { applicationApi } from '@/api/application';
import { tenantApi } from '@/api/tenant';
import { handleApiResult } from '@/utils/request';
import { IResourceInfo, IApplicationInfo, ITenantInfo } from '@/types/base';

const dialog = useDialog();
const message = useMessage();

const router = useRouter();
const loading = ref(false);
const resources = ref<IResourceInfo[]>([]);
const searchKey = ref('');
const selectedTenantId = ref<number | null>(null);
const selectedApplicationId = ref<number | null>(null);
const applications = ref<IApplicationInfo[]>([]);
const tenants = ref<ITenantInfo[]>([]);
const tenantApplications = ref<IApplicationInfo[]>([]);

const tenantOptions = computed(() => {
  return tenants.value
    .filter(tenant => tenant.id !== undefined)
    .map(tenant => ({
      label: tenant.name || `租户 ${tenant.id}`,
      value: tenant.id!
    }));
});

const applicationOptions = computed(() => {
  // 如果选择了租户，只显示该租户下的应用
  const appsToShow = selectedTenantId.value 
    ? tenantApplications.value 
    : applications.value;
  
  return appsToShow
    .filter(app => app.id !== undefined)
    .map(app => ({
      label: app.name || `应用 ${app.id}`,
      value: app.id!
    }));
});

const pagination = reactive({
  page: 1,
  pageSize: 10,
  showSizePicker: true,
  pageSizes: [10, 20, 50],
  itemCount: 0,
  pageCount: 0,
  showQuickJumper: true,
  onChange: (page: number) => {
    pagination.page = page;
    loadResources();
  },
  onUpdatePageSize: (size: number) => {
    pagination.pageSize = size;
    pagination.page = 1;
    loadResources();
  },
  prefix: () => {
    const count = pagination.itemCount || 0;
    const totalPages = count > 0 ? Math.ceil(count / pagination.pageSize) : 0;
    return `共 ${count} 条，共 ${totalPages} 页`;
  }
});

const getResourceTypeLabel = (type?: string) => {
  const typeMap: Record<string, string> = {
    '20': '菜单',
    '40': '按钮',
    '50': '字段',
    '60': '数据'
  };
  return typeMap[type || ''] || type || '-';
};

const getResourceTypeTagType = (type?: string): 'error' | 'default' | 'info' | 'warning' | 'primary' | 'success' => {
  const typeMap: Record<string, 'error' | 'default' | 'info' | 'warning' | 'primary' | 'success'> = {
    '20': 'info',
    '40': 'warning',
    '50': 'success',
    '60': 'default'
  };
  return typeMap[type || ''] || 'default';
};

const columns: DataTableColumns<IResourceInfo> = [
  { title: 'ID', key: 'id', width: 80 },
  { title: '资源代码', key: 'code', width: 150 },
  { title: '资源名称', key: 'name', width: 150 },
  {
    title: '资源类型',
    key: 'resource_type',
    width: 100,
    render: (row) => {
      return h(
        NTag,
        { type: getResourceTypeTagType(row.resource_type) },
        { default: () => getResourceTypeLabel(row.resource_type) }
      );
    }
  },
  { title: '路径', key: 'path', width: 200, ellipsis: { tooltip: true } },
  { title: '父级ID', key: 'parent_id', width: 100 },
  {
    title: '状态',
    key: 'state',
    width: 80,
    render: (row) => {
      return h(
        NTag,
        { type: row.state ? 'success' : 'error' },
        { default: () => (row.state ? '启用' : '禁用') }
      );
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 120,
    fixed: 'right',
    render: (row) => {
      return h('div', { class: 'flex gap-1' }, [
        h(
          NTooltip,
          { trigger: 'hover', placement: 'top' },
          {
            trigger: () => h(
              NButtonComponent,
              {
                size: 'small',
                type: 'primary',
                onClick: () => handleEdit(row.id!),
                quaternary: true,
                circle: true
              },
              {
                icon: () => h(NIcon, { component: PencilOutline, size: 16 })
              }
            ),
            default: () => '编辑'
          }
        ),
        h(
          NTooltip,
          { trigger: 'hover', placement: 'top' },
          {
            trigger: () => h(
              NButtonComponent,
              {
                size: 'small',
                type: 'error',
                onClick: () => handleDelete(row.id!),
                quaternary: true,
                circle: true
              },
              {
                icon: () => h(NIcon, { component: TrashOutline, size: 16 })
              }
            ),
            default: () => '删除'
          }
        )
      ]);
    }
  }
];

const loadTenants = async () => {
  try {
    const response = await tenantApi.getTenantList({ page_size: 100 });
    const data = handleApiResult(response);
    if (data) {
      tenants.value = data.list || [];
    }
  } catch (error: any) {
    console.error('加载租户列表失败', error);
  }
};

const loadApplications = async () => {
  try {
    const response = await applicationApi.getApplicationList({ page_size: 100 });
    const data = handleApiResult(response);
    if (data) {
      applications.value = data.list || [];
    }
  } catch (error: any) {
    console.error('加载应用列表失败', error);
  }
};

const loadTenantApplications = async (tenantId: number) => {
  try {
    const response = await tenantApi.getTenantApplications(tenantId);
    const data = handleApiResult(response);
    if (data) {
      tenantApplications.value = data || [];
    }
  } catch (error: any) {
    console.error('加载租户应用列表失败', error);
    tenantApplications.value = [];
  }
};

const loadResources = async () => {
  loading.value = true;
  try {
    const response = await resourceApi.getResourceList({
      page_size: pagination.pageSize,
      cursor: pagination.page > 1 ? pagination.page : undefined,
      application_id: selectedApplicationId.value || undefined,
      tenant_id: selectedTenantId.value || undefined,
      search_key: searchKey.value || undefined
    });
    const data = handleApiResult(response);
    if (data) {
      // 直接使用后端返回的数据和总数（搜索由后端处理）
      resources.value = data.list || [];
      // 使用 ?? 而不是 ||，避免 total 为 0 时被误判
      const total = typeof data.total === 'number' ? data.total : 0;
      // 先设置 itemCount，然后计算 pageCount
      pagination.itemCount = total;
      pagination.pageCount = total > 0 ? Math.ceil(total / pagination.pageSize) : 0;
    } else {
      // 如果没有数据，重置分页
      resources.value = [];
      pagination.itemCount = 0;
      pagination.pageCount = 0;
    }
  } catch (error: any) {
    message.error(error.message || '加载资源列表失败');
    resources.value = [];
    pagination.itemCount = 0;
    pagination.pageCount = 0;
  } finally {
    loading.value = false;
  }
};

const handleTenantChange = async () => {
  selectedApplicationId.value = null; // 清空应用选择
  tenantApplications.value = [];
  pagination.page = 1;
  
  if (selectedTenantId.value) {
    await loadTenantApplications(selectedTenantId.value);
  }
  
  loadResources();
};

const handleApplicationChange = () => {
  pagination.page = 1;
  loadResources();
};

const handleSearch = () => {
  pagination.page = 1;
  loadResources();
};

const handleCreate = () => {
  router.push({ name: 'ResourceDetail', params: { id: 'new' } });
};

const handleEdit = (id: number) => {
  router.push({ name: 'ResourceDetail', params: { id } });
};

const handleDelete = async (id: number) => {
  dialog.warning({
    title: '确认删除',
    content: '确定要删除该资源吗？',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await resourceApi.deleteResource(id);
        message.success('删除成功');
        await loadResources();
      } catch (error: any) {
        message.error(error.message || '删除失败');
      }
    }
  });
};

onMounted(() => {
  loadTenants();
  loadApplications();
  loadResources();
});
</script>

