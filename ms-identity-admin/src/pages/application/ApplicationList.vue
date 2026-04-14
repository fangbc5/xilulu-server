<template>
  <div>
    <h1 class="text-2xl font-bold mb-4">应用管理</h1>
    <n-card>
      <template #header>
        <div class="flex justify-between items-center">
          <span>应用列表</span>
          <div class="flex gap-2">
            <n-input
              v-model:value="searchKey"
              placeholder="搜索应用标识或名称"
              clearable
              style="width: 300px"
              @keyup.enter="handleSearch"
            />
            <n-button @click="handleSearch">搜索</n-button>
            <n-button type="primary" @click="handleCreate">创建应用</n-button>
          </div>
        </div>
      </template>
      <n-data-table
        :columns="columns"
        :data="applications"
        :loading="loading"
        :pagination="pagination"
        remote
      />
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h } from 'vue';
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
  NTag
} from 'naive-ui';
import { PencilOutline, TrashOutline } from '@vicons/ionicons5';
import { applicationApi } from '@/api/application';
import { handleApiResult } from '@/utils/request';
import { IApplicationInfo } from '@/types/base';

const dialog = useDialog();
const message = useMessage();

const router = useRouter();
const loading = ref(false);
const applications = ref<IApplicationInfo[]>([]);
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
    loadApplications();
  },
  onUpdatePageSize: (pageSize: number) => {
    pagination.value.pageSize = pageSize;
    pagination.value.page = 1;
    loadApplications();
  },
  prefix: ({ itemCount }: { itemCount?: number }) => {
    const count = itemCount || 0;
    const totalPages = count > 0 ? Math.ceil(count / pagination.value.pageSize) : 0;
    return `共 ${count} 条，共 ${totalPages} 页`;
  }
});

const columns: DataTableColumns<IApplicationInfo> = [
  { title: 'ID', key: 'id', width: 80 },
  { title: '应用标识', key: 'app_key', width: 150 },
  { title: '应用名称', key: 'name', width: 150 },
  {
    title: '应用类型',
    key: 'type',
    width: 100,
    render: (row) => {
      return h(
        NTag,
        { type: row.type === '10' ? 'info' : 'warning' },
        { default: () => row.type === '10' ? '自建应用' : '第三方应用' }
      );
    }
  },
  { title: '版本', key: 'version', width: 100 },
  { title: '应用地址', key: 'url', width: 200, ellipsis: { tooltip: true } },
  {
    title: '公共应用',
    key: 'is_general',
    width: 100,
    render: (row) => {
      return h(
        NTag,
        { type: row.is_general ? 'success' : 'default' },
        { default: () => row.is_general ? '是' : '否' }
      );
    }
  },
  {
    title: '可见',
    key: 'is_visible',
    width: 80,
    render: (row) => {
      return h(
        NTag,
        { type: row.is_visible ? 'success' : 'error' },
        { default: () => row.is_visible ? '是' : '否' }
      );
    }
  },
  { title: '排序', key: 'sort_value', width: 80 },
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

const loadApplications = async () => {
  loading.value = true;
  try {
    const response = await applicationApi.getApplicationList({
      page_size: pagination.value.pageSize,
      cursor: pagination.value.page > 1 ? pagination.value.page : undefined,
      search_key: searchKey.value || undefined
    });
    const data = handleApiResult(response);
    if (data) {
      applications.value = data.list || [];
      const total = data.total || 0;
      pagination.value.itemCount = total;
      pagination.value.pageCount = total > 0 ? Math.ceil(total / pagination.value.pageSize) : 0;
    }
  } catch (error: any) {
    message.error(error.message || '加载应用列表失败');
  } finally {
    loading.value = false;
  }
};

const handleSearch = () => {
  pagination.value.page = 1;
  loadApplications();
};

const handleCreate = () => {
  router.push({ name: 'ApplicationDetail', params: { id: 'new' } });
};

const handleEdit = (id: number) => {
  router.push({ name: 'ApplicationDetail', params: { id } });
};

const handleDelete = async (id: number) => {
  dialog.warning({
    title: '确认删除',
    content: '确定要删除该应用吗？',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await applicationApi.deleteApplication(id);
        message.success('删除成功');
        await loadApplications();
      } catch (error: any) {
        message.error(error.message || '删除失败');
      }
    }
  });
};

onMounted(() => {
  loadApplications();
});
</script>

