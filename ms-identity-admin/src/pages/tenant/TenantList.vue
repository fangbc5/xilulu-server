<template>
  <div>
    <h1 class="text-2xl font-bold mb-4">租户管理</h1>
    <n-card>
      <template #header>
        <div class="flex justify-between items-center">
          <span>租户列表</span>
          <div class="flex gap-2">
            <n-input
              v-model:value="searchKey"
              placeholder="搜索租户名称、联系人、联系电话"
              clearable
              style="width: 300px"
              @keyup.enter="handleSearch"
            />
            <n-button @click="handleSearch">搜索</n-button>
            <n-button type="primary" @click="handleCreate">
              <template #icon>
                <n-icon :component="AddOutline" />
              </template>
              创建租户
            </n-button>
          </div>
        </div>
      </template>
      <n-data-table
        :columns="columns"
        :data="tenants"
        :loading="loading"
        :pagination="pagination"
        :bordered="false"
        remote
      />
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, h, onMounted } from 'vue';
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
  NTag,
  NTooltip
} from 'naive-ui';
import { AddOutline, PencilOutline, TrashOutline } from '@vicons/ionicons5';
import { tenantApi } from '@/api/tenant';
import { handleApiResult } from '@/utils/request';
import { ITenantInfo } from '@/types/base';

const dialog = useDialog();
const message = useMessage();

const router = useRouter();
const loading = ref(false);
const tenants = ref<ITenantInfo[]>([]);
const searchKey = ref('');

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
    loadTenants();
  },
  onUpdatePageSize: (pageSize: number) => {
    pagination.value.pageSize = pageSize;
    pagination.value.page = 1;
    loadTenants();
  },
  prefix: ({ itemCount }: { itemCount?: number }) => {
    const count = itemCount || 0;
    const totalPages = count > 0 ? Math.ceil(count / pagination.value.pageSize) : 0;
    return `共 ${count} 条，共 ${totalPages} 页`;
  }
});

const columns: DataTableColumns<ITenantInfo> = [
  { title: 'ID', key: 'id', width: 80 },
  { title: '租户名称', key: 'name', width: 150 },
  { title: '联系人', key: 'contact_name', width: 120 },
  { title: '联系电话', key: 'contact_mobile', width: 130 },
  {
    title: '状态',
    key: 'status',
    width: 100,
    render: (row) => {
      return h(
        NTag,
        { type: row.status === 0 ? 'success' : 'error' },
        { default: () => (row.status === 0 ? '正常' : '停用') }
      );
    }
  },
  {
    title: '套餐ID',
    key: 'package_id',
    width: 100
  },
  {
    title: '账户数',
    key: 'account_count',
    width: 100
  },
  {
    title: '过期时间',
    key: 'expire_time',
    width: 180,
    render: (row) => {
      return row.expire_time
        ? new Date(row.expire_time).toLocaleString('zh-CN')
        : '-';
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 100,
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
  loading.value = true;
  try {
    const response = await tenantApi.getTenantList({
      page_size: pagination.value.pageSize,
      cursor: pagination.value.page > 1 ? pagination.value.page : undefined,
      search_key: searchKey.value || undefined
    });
    const data = handleApiResult(response);
    if (data) {
      tenants.value = data.list || [];
      const total = data.total || 0;
      pagination.value.itemCount = total;
      pagination.value.pageCount = total > 0 ? Math.ceil(total / pagination.value.pageSize) : 0;
    }
  } catch (error: any) {
    message.error(error.message || '加载租户列表失败');
  } finally {
    loading.value = false;
  }
};

const handleSearch = () => {
  pagination.value.page = 1;
  loadTenants();
};

const handleCreate = () => {
  router.push({ name: 'TenantDetail', params: { id: 'new' } });
};

const handleEdit = (id: number) => {
  router.push({ name: 'TenantDetail', params: { id } });
};

const handleDelete = async (id: number) => {
  dialog.warning({
    title: '确认删除',
    content: '确定要删除该租户吗？',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await tenantApi.deleteTenant(id);
        message.success('删除成功');
        await loadTenants();
      } catch (error: any) {
        message.error(error.message || '删除失败');
      }
    }
  });
};

onMounted(() => {
  loadTenants();
});
</script>

