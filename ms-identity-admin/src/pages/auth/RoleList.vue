<template>
  <div>
    <h1 class="text-2xl font-bold mb-4">角色管理</h1>
    <n-card>
      <template #header>
        <div class="flex justify-between items-center">
          <span>角色列表</span>
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
            <n-button type="primary" @click="handleCreate">创建角色</n-button>
          </div>
        </div>
      </template>
      <n-data-table
        :columns="columns"
        :data="roles"
        :loading="loading"
        :pagination="pagination"
        remote
      />
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, computed } from 'vue';
import { useRouter } from 'vue-router';
import {
  NCard,
  NButton,
  NDataTable,
  DataTableColumns,
  useDialog,
  useMessage,
  NButton as NButtonComponent,
  NIcon,
  NTag,
  NTooltip,
  NSelect
} from 'naive-ui';
import { PencilOutline, TrashOutline } from '@vicons/ionicons5';
import { roleApi } from '@/api/role';
import { IRoleInfo } from '@/types/base';
import { tenantApi } from '@/api/tenant';
import { handleApiResult } from '@/utils/request';
import { ITenantInfo } from '@/types/base';

const dialog = useDialog();
const message = useMessage();

const router = useRouter();
const loading = ref(false);
const roles = ref<IRoleInfo[]>([]);
const selectedTenantId = ref<number | null>(null);
const tenants = ref<ITenantInfo[]>([]);

const tenantOptions = computed(() => {
  return tenants.value
    .filter(tenant => tenant.id !== undefined)
    .map(tenant => ({
      label: tenant.name || `租户 ${tenant.id}`,
      value: tenant.id!
    }));
});

const pagination = ref({
  page: 1,
  pageSize: 10,
  showSizePicker: true,
  pageSizes: [10, 20, 50],
  itemCount: 0,
  pageCount: 0,
  showQuickJumper: true,
  onChange: (page: number) => {
    pagination.value.page = page;
    loadRoles();
  },
  onUpdatePageSize: (pageSize: number) => {
    pagination.value.pageSize = pageSize;
    pagination.value.page = 1;
    loadRoles();
  },
  prefix: ({ itemCount }: { itemCount?: number }) => {
    const count = itemCount || 0;
    const totalPages = count > 0 ? Math.ceil(count / pagination.value.pageSize) : 0;
    return `共 ${count} 条，共 ${totalPages} 页`;
  }
});

const columns: DataTableColumns<IRoleInfo> = [
  { title: 'ID', key: 'id', width: 80 },
  { title: '角色编码', key: 'code', width: 150 },
  { title: '角色名称', key: 'name', width: 150 },
  { title: '租户ID', key: 'tenant_id', width: 100 },
  {
    title: '状态',
    key: 'state',
    width: 100,
    render: (row) => {
      return h(
        NTag,
        { type: row.state ? 'success' : 'error' },
        { default: () => (row.state ? '启用' : '禁用') }
      );
    }
  },
  { title: '备注', key: 'remarks', width: 200, ellipsis: { tooltip: true } },
  {
    title: '操作',
    key: 'actions',
    width: 120,
    fixed: 'right',
    render: (row) => {
      const children: any[] = [];
      children.push(
        h(
          NTooltip,
          { trigger: 'hover', placement: 'top' },
          {
            trigger: () =>
              h(
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
        )
      );
      children.push(
        h(
          NTooltip,
          { trigger: 'hover', placement: 'top' },
          {
            trigger: () =>
              h(
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
      );
      return h('div', { class: 'flex gap-1' }, children);
    }
  }
];

const loadTenants = async () => {
  try {
    const response = await tenantApi.getTenantList({
      page_size: 100,
      cursor: undefined
    });
    const data = handleApiResult(response);
    if (data) {
      tenants.value = data.list || [];
    }
  } catch (error: any) {
    console.error('加载租户列表失败', error);
  }
};

const loadRoles = async () => {
  loading.value = true;
  try {
    const response = await roleApi.getRoleList({
      page_size: pagination.value.pageSize,
      cursor: pagination.value.page > 1 ? pagination.value.page : undefined,
      tenant_id: selectedTenantId.value || undefined
    });
    const data = handleApiResult(response);
    if (data) {
      roles.value = data.list || [];
      const total = data.total || 0;
      pagination.value.itemCount = total;
      pagination.value.pageCount = total > 0 ? Math.ceil(total / pagination.value.pageSize) : 0;
    }
  } catch (error: any) {
    message.error(error.message || '加载角色列表失败');
    roles.value = [];
    pagination.value.itemCount = 0;
    pagination.value.pageCount = 0;
  } finally {
    loading.value = false;
  }
};

const handleTenantChange = () => {
  pagination.value.page = 1;
  loadRoles();
};

const handleCreate = () => {
  router.push({ name: 'RoleDetail', params: { id: 'new' } });
};

const handleEdit = (id: number) => {
  router.push({ name: 'RoleDetail', params: { id } });
};

const handleDelete = (id: number) => {
  dialog.warning({
    title: '确认删除',
    content: '确定要删除这个角色吗？删除后无法恢复。',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await roleApi.deleteRole(id);
        message.success('删除成功');
        loadRoles();
      } catch (error: any) {
        message.error(error.message || '删除失败');
      }
    }
  });
};

onMounted(async () => {
  await loadTenants();
  await loadRoles();
});
</script>
