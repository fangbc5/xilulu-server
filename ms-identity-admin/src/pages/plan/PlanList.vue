<template>
  <div>
    <h1 class="text-2xl font-bold mb-4">套餐管理</h1>
    <n-card>
      <template #header>
        <div class="flex justify-between items-center">
          <span>套餐列表</span>
          <div class="flex gap-2">
            <n-input
              v-model:value="searchKey"
              placeholder="搜索套餐名称、类型"
              clearable
              style="width: 300px"
              @keyup.enter="handleSearch"
            />
            <n-button @click="handleSearch">搜索</n-button>
            <n-button type="primary" @click="handleCreate">创建套餐</n-button>
          </div>
        </div>
      </template>
      <n-data-table
        :columns="columns"
        :data="plans"
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
  NTag,
  NTooltip
} from 'naive-ui';
import { PencilOutline, TrashOutline, SettingsOutline } from '@vicons/ionicons5';
import { planApi } from '@/api/plan';
import { handleApiResult } from '@/utils/request';
import { IPlanInfo } from '@/types/base';

const dialog = useDialog();
const message = useMessage();
const router = useRouter();

const loading = ref(false);
const plans = ref<IPlanInfo[]>([]);
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
    loadPlans();
  },
  onUpdatePageSize: (pageSize: number) => {
    pagination.value.pageSize = pageSize;
    pagination.value.page = 1;
    loadPlans();
  },
  prefix: ({ itemCount }: { itemCount?: number }) => {
    const count = itemCount || 0;
    const totalPages = count > 0 ? Math.ceil(count / pagination.value.pageSize) : 0;
    return `共 ${count} 条，共 ${totalPages} 页`;
  }
});

const columns: DataTableColumns<IPlanInfo> = [
  { title: 'ID', key: 'id', width: 80 },
  { title: '套餐名称', key: 'name', width: 150 },
  {
    title: '套餐类型',
    key: 'type',
    width: 120,
    render: (row) => {
      const typeMap: Record<string, { label: string; type: string }> = {
        personal: { label: '个人版', type: 'info' },
        team: { label: '团队版', type: 'success' },
        enterprise: { label: '企业版', type: 'warning' }
      };
      const typeInfo = typeMap[row.type] || { label: row.type, type: 'default' };
      return h(NTag, { type: typeInfo.type as any }, { default: () => typeInfo.label });
    }
  },
  {
    title: '价格',
    key: 'price',
    width: 120,
    render: (row) => {
      const price = row.price || '0.00';
      return `${price} 元`;
    }
  },
  { title: '计费周期', key: 'billing_cycle', width: 120 },
  {
    title: '状态',
    key: 'is_active',
    width: 100,
    render: (row) => {
      return h(
        NTag,
        { type: row.is_active ? 'success' : 'error' },
        { default: () => (row.is_active ? '启用' : '禁用') }
      );
    }
  },
  {
    title: '排序',
    key: 'sort_order',
    width: 80
  },
  {
    title: '描述',
    key: 'description',
    ellipsis: {
      tooltip: true
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 140,
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
                type: 'info',
                onClick: () => handleManageEntitlements(row.id!),
                quaternary: true,
                circle: true
              },
              {
                icon: () => h(NIcon, { component: SettingsOutline, size: 16 })
              }
            ),
            default: () => '管理权益'
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

const loadPlans = async () => {
  loading.value = true;
  try {
    const response = await planApi.getPlanList({
      page_size: pagination.value.pageSize,
      cursor: pagination.value.page > 1 ? pagination.value.page : undefined,
      search_key: searchKey.value || undefined
    });
    const data = handleApiResult(response);
    if (data) {
      plans.value = data.list || [];
      const total = data.total || 0;
      pagination.value.itemCount = total;
      pagination.value.pageCount = total > 0 ? Math.ceil(total / pagination.value.pageSize) : 0;
    }
  } catch (error: any) {
    message.error(error.message || '加载套餐列表失败');
  } finally {
    loading.value = false;
  }
};

const handleSearch = () => {
  pagination.value.page = 1;
  loadPlans();
};

const handleCreate = () => {
  router.push({ name: 'PlanDetail', params: { id: 'new' } });
};

const handleEdit = (id: number) => {
  router.push({ name: 'PlanDetail', params: { id } });
};

const handleManageEntitlements = (id: number) => {
  router.push({ name: 'PlanDetail', params: { id, tab: 'entitlements' } });
};

const handleDelete = async (id: number) => {
  dialog.warning({
    title: '确认删除',
    content: '确定要删除该套餐吗？删除后无法恢复。',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await planApi.deletePlan(id);
        message.success('删除成功');
        await loadPlans();
      } catch (error: any) {
        message.error(error.message || '删除失败');
      }
    }
  });
};

onMounted(() => {
  loadPlans();
});
</script>
