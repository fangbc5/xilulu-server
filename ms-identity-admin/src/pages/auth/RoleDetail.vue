<template>
  <div class="detail-container">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="flex items-center gap-3">
        <n-button quaternary circle @click="handleBack">
          <template #icon>
            <n-icon :component="ArrowBackOutline" />
          </template>
        </n-button>
        <h1 class="text-2xl font-bold m-0">{{ isNew ? '创建角色' : '角色详情' }}</h1>
        <n-tag v-if="!isNew && form.code" type="info" size="small" round>{{ form.code }}</n-tag>
      </div>
    </div>
    
    <!-- 内容区域 -->
    <div class="content-wrapper">
      <!-- 基本信息 -->
      <n-card class="info-card" :bordered="false">
        <template #header>
          <div class="card-header">
            <div class="flex items-center gap-2">
              <n-icon size="18" :component="PersonOutline" />
              <span class="font-semibold text-base">基本信息</span>
      </div>
      <div class="flex gap-2">
        <n-button @click="handleBack">取消</n-button>
        <n-button type="primary" @click="handleSave" :loading="saving">保存</n-button>
      </div>
        </div>
      </template>
      <n-form ref="formRef" :model="form" :rules="rules" label-placement="left" label-width="100">
        <n-grid :cols="2" :x-gap="24" :y-gap="16">
          <n-gi>
            <n-form-item path="code" label="角色编码">
              <n-input v-model:value="form.code" :disabled="!isNew" placeholder="请输入角色编码" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="name" label="角色名称">
              <n-input v-model:value="form.name" placeholder="请输入角色名称" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="tenant_id" label="租户">
              <n-select
                v-model:value="form.tenant_id"
                :options="tenantOptions"
                placeholder="请选择租户"
                filterable
                :disabled="!isNew"
              />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="state" label="状态">
              <n-switch v-model:value="form.state" />
              <span class="ml-2">{{ form.state ? '启用' : '禁用' }}</span>
            </n-form-item>
          </n-gi>
          <n-gi :span="2">
            <n-form-item path="remarks" label="备注">
              <n-input
                v-model:value="form.remarks"
                type="textarea"
                placeholder="请输入备注"
                :rows="3"
              />
            </n-form-item>
          </n-gi>
        </n-grid>
      </n-form>
    </n-card>

    <!-- 权限管理（仅在非新建时显示） -->
    <n-card v-if="!isNew" class="info-card" :bordered="false">
      <template #header>
        <div class="flex items-center gap-2">
          <n-icon size="18" :component="ShieldCheckmarkOutline" />
          <span class="font-semibold text-base">权限管理</span>
        </div>
      </template>
      <div class="mb-4">
        <div class="flex items-center gap-4 mb-4">
          <n-select
            v-model:value="selectedApplicationId"
            :options="applicationOptions"
            placeholder="选择应用"
            filterable
            style="width: 300px"
            @update:value="handleApplicationChange"
          />
          <n-button type="primary" @click="openAddResourceDialog" :disabled="!selectedApplicationId">
            <template #icon>
              <n-icon :component="AddOutline" />
            </template>
            分配权限
          </n-button>
        </div>
      </div>
      
      <!-- 已分配的权限列表（树表） -->
      <n-data-table
        :columns="resourceColumns"
        :data="assignedResourcesTree"
        :loading="resourcesLoading"
        :bordered="false"
        :default-expand-all="false"
        :pagination="false"
        :row-key="(row: IResourceInfo) => row.id ?? ''"
        :expanded-row-keys="expandedRowKeys"
        @update:expanded-row-keys="handleExpandedRowKeysUpdate"
      />
    </n-card>
    </div>

    <!-- 添加资源对话框 -->
    <n-modal v-model:show="showAddResourceDialog" preset="dialog" title="分配权限" positive-text="确定" negative-text="取消" @positive-click="handleAddResource">
      <div style="max-height: 500px; overflow-y: auto;">
        <n-tree
          v-if="allResourcesForDialog.length > 0"
          :data="dialogResourceTreeData"
          :checkable="true"
          :checked-keys="dialogCheckedKeys"
          check-strategy="all"
          @update:checked-keys="handleDialogResourceSelection"
        />
        <n-empty v-else description="该应用下暂无权限" />
      </div>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, h } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import {
  NCard,
  NForm,
  NFormItem,
  NInput,
  NButton,
  FormInst,
  useMessage,
  useDialog,
  NModal,
  NDataTable,
  DataTableColumns,
  NSelect,
  NTag,
  NIcon,
  NTree,
  NEmpty,
  NGrid,
  NGi,
  NSwitch
} from 'naive-ui';
import { ArrowBackOutline, PersonOutline, ShieldCheckmarkOutline, AddOutline, TrashOutline } from '@vicons/ionicons5';
import { roleApi } from '@/api/role';
import { resourceApi } from '@/api/resource';
import { applicationApi } from '@/api/application';
import { tenantApi } from '@/api/tenant';
import { handleApiResult } from '@/utils/request';
import { ITenantInfo, IApplicationInfo, IResourceInfo } from '@/types/base';

const message = useMessage();
const dialog = useDialog();

const route = useRoute();
const router = useRouter();
const formRef = ref<FormInst | null>(null);

const roleId = route.params.id as string;
const isNew = computed(() => roleId === 'new');
const saving = ref(false);

const form = ref({
  code: '',
  name: '',
  tenant_id: null as number | null,
  state: true,
  remarks: ''
});

const rules = {
  code: {
    required: true,
    message: '请输入角色编码',
    trigger: 'blur'
  },
  name: {
    required: true,
    message: '请输入角色名称',
    trigger: 'blur'
  },
  tenant_id: {
      required: true,
    message: '请选择租户',
    trigger: 'change',
    validator: (_rule: any, value: number | null) => {
      if (value === null || value === undefined) {
          return new Error('请选择租户');
        }
        return true;
    }
  }
};

// 租户选项
const tenantOptions = ref<Array<{ label: string; value: number }>>([]);

// 权限管理相关
const selectedApplicationId = ref<number | null>(null);
const applicationOptions = ref<Array<{ label: string; value: number }>>([]);
const assignedResources = ref<IResourceInfo[]>([]);
const availableResources = ref<IResourceInfo[]>([]);
const allResourcesForDialog = ref<IResourceInfo[]>([]); // 对话框显示的所有资源（包括已分配和未分配）
const resourcesLoading = ref(false);
const showAddResourceDialog = ref(false);
const selectedResourceIds = ref<Array<string | number>>([]);
const dialogSelectedResourceIds = ref<Array<string | number>>([]); // 对话框中选中的资源ID（仅未分配的）
const expandedRowKeys = ref<Array<string | number>>([]);

// 加载租户列表
const loadTenants = async () => {
  try {
    const response = await tenantApi.getTenantList({ page_size: 100 });
    const data = handleApiResult(response);
    if (data) {
      tenantOptions.value = (data.list || []).map((t: ITenantInfo) => ({
        label: t.name,
        value: t.id!
      }));
    }
  } catch (error: any) {
    message.error(error.message || '加载租户列表失败');
  }
};

// 加载应用列表
const loadApplications = async () => {
  try {
    const response = await applicationApi.getApplicationList({ page_size: 100 });
    const data = handleApiResult(response);
    if (data) {
      applicationOptions.value = (data.list || []).map((app: IApplicationInfo) => ({
        label: app.name || '',
        value: app.id!
      }));
    }
  } catch (error: any) {
    message.error(error.message || '加载应用列表失败');
  }
};

// 加载角色信息
const loadRole = async () => {
  if (isNew.value) return;
  try {
    const response = await roleApi.getRole(Number(roleId));
    const data = handleApiResult(response);
    if (data) {
      form.value = {
        code: data.code || '',
        name: data.name || '',
        tenant_id: data.tenant_id || null,
        state: data.state === true || (typeof data.state === 'number' && data.state === 1),
        remarks: data.remarks || ''
      };
    }
    await loadRoleResources();
    await loadApplications();
  } catch (error: any) {
    message.error(error.message || '加载角色信息失败');
  }
};

// 加载角色已分配的权限
const loadRoleResources = async () => {
  if (isNew.value || !roleId) return;
  resourcesLoading.value = true;
  try {
    const response = await roleApi.getRoleResources(Number(roleId));
    const data = handleApiResult(response);
    assignedResources.value = (data || []) as IResourceInfo[];
  } catch (error: any) {
    message.error(error.message || '加载权限列表失败');
  } finally {
    resourcesLoading.value = false;
  }
};

// 加载可用资源（未分配的）
const loadAvailableResources = async () => {
  if (!selectedApplicationId.value || !roleId) {
    message.warning('请先选择应用');
    return;
  }
  try {
    await loadRoleResources(); // 确保 assignedResources 是最新的
    const response = await resourceApi.getApplicationResources(selectedApplicationId.value);
    const data = handleApiResult(response);
    if (data) {
      const assignedResourceIds = new Set(assignedResources.value.map(resource => resource.id).filter((id): id is number => id !== undefined));
      availableResources.value = data.filter(resource =>
        resource.id !== undefined && !assignedResourceIds.has(resource.id)
      );
      if (availableResources.value.length === 0) {
        message.info('该应用下暂无未分配的权限');
      }
    } else {
      availableResources.value = [];
    }
  } catch (error: any) {
    message.error(error.message || '加载可用资源失败');
    availableResources.value = [];
  }
};

// 加载所有资源（用于分配权限对话框，包括已分配和未分配的）
const loadAllResourcesForDialog = async () => {
  if (!selectedApplicationId.value || !roleId) {
    return;
  }
  try {
    await loadRoleResources(); // 确保 assignedResources 是最新的
    const response = await resourceApi.getApplicationResources(selectedApplicationId.value);
    const data = handleApiResult(response);
    allResourcesForDialog.value = data || [];
  } catch (error: any) {
    message.error(error.message || '加载资源失败');
    allResourcesForDialog.value = [];
  }
};

// 将资源转换为树形结构（用于已分配权限的树表显示）
const assignedResourcesTree = computed(() => {
  if (assignedResources.value.length === 0) return [];

  const resourceMap = new Map<number, IResourceInfo & { children?: IResourceInfo[] }>();
  const rootResources: (IResourceInfo & { children?: IResourceInfo[] })[] = [];

  assignedResources.value.forEach((resource: IResourceInfo) => {
    if (resource.id !== undefined) {
      resourceMap.set(resource.id, { ...resource, children: [] });
    }
  });

  const buildTree = (resource: IResourceInfo & { children?: IResourceInfo[] }) => {
    if (resource.parent_id === 0) {
      return true;
    }
    const parent = resourceMap.get(resource.parent_id);
    if (parent) {
      if (!parent.children) parent.children = [];
      parent.children.push(resource);
      return false;
    }
    return true;
  };

  const sortedResources = [...assignedResources.value].sort((a, b) => {
    const aSort = a.sort_value ?? 0;
    const bSort = b.sort_value ?? 0;
    if (aSort !== bSort) return aSort - bSort;
    return (a.id || 0) - (b.id || 0);
  });

  sortedResources.forEach(resource => {
    if (resource.id !== undefined) {
      const node = resourceMap.get(resource.id)!;
      if (buildTree(node)) {
        rootResources.push(node);
      }
    }
  });

  const sortChildren = (node: IResourceInfo & { children?: IResourceInfo[] }) => {
    if (node.children && node.children.length > 0) {
      node.children.sort((a: IResourceInfo, b: IResourceInfo) => {
        const aSort = a.sort_value ?? 0;
        const bSort = b.sort_value ?? 0;
        if (aSort !== bSort) return aSort - bSort;
        return (a.id || 0) - (b.id || 0);
      });
      node.children.forEach((child: IResourceInfo) => sortChildren(child));
    }
  };
  rootResources.forEach(root => sortChildren(root));
  return rootResources;
});

// 将资源转换为树形结构（用于选择对话框的树组件）
interface ResourceTreeNode {
  key: number;
  label: string;
  children?: ResourceTreeNode[];
  resource_type?: string;
  parent_id?: number;
  sort_value?: number;
}

const convertToTreeData = (resources: IResourceInfo[]): ResourceTreeNode[] => {
  const resourceMap = new Map<number, ResourceTreeNode>();
  const rootNodes: ResourceTreeNode[] = [];

  resources.forEach(resource => {
    if (resource.id !== undefined) {
      resourceMap.set(resource.id, {
        key: resource.id,
        label: `${resource.name} (${resource.code})`,
        children: [],
        resource_type: resource.resource_type,
        parent_id: resource.parent_id,
        sort_value: resource.sort_value,
      });
    }
  });

  const sortedResources = [...resources].sort((a, b) => {
    const aSort = a.sort_value ?? 0;
    const bSort = b.sort_value ?? 0;
    if (aSort !== bSort) return aSort - bSort;
    return (a.id || 0) - (b.id || 0);
  });

  sortedResources.forEach(resource => {
    if (resource.id !== undefined) {
      const node = resourceMap.get(resource.id)!;
      if (resource.parent_id === 0) {
        rootNodes.push(node);
      } else {
        const parent = resourceMap.get(resource.parent_id);
        if (parent) {
          parent.children!.push(node);
        } else {
          rootNodes.push(node);
        }
      }
    }
  });

  const sortTreeNodes = (nodes: ResourceTreeNode[]) => {
    nodes.sort((a, b) => {
      const aSort = a.sort_value ?? 0;
      const bSort = b.sort_value ?? 0;
      if (aSort !== bSort) return aSort - bSort;
      return (a.key || 0) - (b.key || 0);
    });
    nodes.forEach(node => {
      if (node.children && node.children.length > 0) {
        sortTreeNodes(node.children);
      }
    });
  };

  sortTreeNodes(rootNodes);
  return rootNodes;
};

const resourceTreeData = computed(() => {
  const treeData = convertToTreeData(availableResources.value);
  // 转换为 TreeOption 格式
  return treeData.map(node => ({
    key: node.key,
    label: node.label,
    children: node.children?.map(child => ({
      key: child.key,
      label: child.label,
      children: child.children
    }))
  })) as any;
});

// 对话框中的资源树形数据（包含所有资源，但只显示有未分配权限的层级）
const dialogResourceTreeData = computed(() => {
  const assignedResourceIds = new Set(assignedResources.value.map(resource => resource.id).filter((id): id is number => id !== undefined));
  
  // 过滤出有未分配权限的资源（包括其父节点）
  const hasUnassignedDescendant = (resource: IResourceInfo): boolean => {
    // 如果当前资源未分配，返回true
    if (resource.id !== undefined && !assignedResourceIds.has(resource.id)) {
      return true;
    }
    // 检查是否有未分配的子节点
    const children = allResourcesForDialog.value.filter(r => r.parent_id === resource.id);
    return children.some(child => hasUnassignedDescendant(child));
  };
  
  // 过滤资源：只保留有未分配权限的资源及其父节点
  const filteredResources = allResourcesForDialog.value.filter(resource => {
    if (resource.parent_id === 0) {
      // 根节点：如果有未分配的后代，则保留
      return hasUnassignedDescendant(resource);
    } else {
      // 非根节点：如果自己未分配，或者有未分配的后代，则保留
      return hasUnassignedDescendant(resource);
    }
  });
  
  const treeData = convertToTreeData(filteredResources);
  // 转换为 TreeOption 格式，并添加 disabled 属性
  const convertNode = (node: ResourceTreeNode): any => {
    const isAssigned = node.key !== undefined && assignedResourceIds.has(node.key);
    return {
      key: node.key,
      label: node.label,
      disabled: isAssigned,
      children: node.children?.map(convertNode)
    };
  };
  
  return treeData.map(convertNode) as any;
});

// 对话框中已勾选的资源（包括已分配的，用于显示）
const dialogCheckedKeys = computed(() => {
  const assignedResourceIds = assignedResources.value.map(resource => resource.id).filter((id): id is number => id !== undefined);
  // 返回已分配的ID和用户新选择的未分配ID
  return [...assignedResourceIds, ...dialogSelectedResourceIds.value.map(id => Number(id))];
});

// 资源表格列定义
const assignedResourceIdsSet = computed(() => new Set(assignedResources.value.map(resource => resource.id).filter((id): id is number => id !== undefined)));

const resourceColumns: DataTableColumns<IResourceInfo & { children?: IResourceInfo[] }> = [
  {
    title: '资源名称',
    key: 'name',
    width: 200,
    render: (row) => row.name
  },
  {
    title: '资源代码',
    key: 'code',
    width: 200,
    render: (row) => row.code
  },
  {
    title: '资源类型',
    key: 'resource_type',
    width: 120,
    render: (row) => {
      const typeMap: Record<string, string> = {
        '20': '菜单',
        '40': '按钮',
        '50': '字段',
        '60': '数据'
      };
      return typeMap[row.resource_type || ''] || row.resource_type || '-';
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 100,
    render: (row) => {
      if (row.id !== undefined && assignedResourceIdsSet.value.has(row.id)) {
        return h(NButton, {
          size: 'small',
          type: 'error',
          quaternary: true,
          onClick: () => handleRemoveResource(row.id!),
          style: { marginRight: '8px' }
        }, {
          default: () => '移除',
          icon: () => h(NIcon, { component: TrashOutline })
        });
      }
      return null;
    }
  }
];

// 处理展开行
const handleExpandedRowKeysUpdate = (keys: Array<string | number>) => {
  expandedRowKeys.value = keys;
};

// 处理应用切换
const handleApplicationChange = async () => {
  await loadRoleResources();
};

// 打开添加资源对话框
const openAddResourceDialog = async () => {
  if (!selectedApplicationId.value) {
    message.warning('请先选择应用');
    return;
  }
  
  if (!roleId || roleId === 'new') {
    message.warning('角色ID不存在');
    return;
  }
  
  dialogSelectedResourceIds.value = [];
  await loadAllResourcesForDialog();
  
  if (allResourcesForDialog.value.length > 0) {
    showAddResourceDialog.value = true;
  } else {
    message.info('该应用下暂无权限');
  }
};

const handleBack = () => {
  router.push({ name: 'Roles' });
};

const handleSave = async () => {
  try {
    await formRef.value?.validate();
  saving.value = true;
    if (isNew.value) {
      const response = await roleApi.createRole({
        code: form.value.code,
        name: form.value.name,
        tenant_id: form.value.tenant_id!,
        state: form.value.state,
        remarks: form.value.remarks || undefined
      });
      const data = handleApiResult(response);
      if (data?.role_id) {
      message.success('创建成功');
        router.replace({ name: 'RoleDetail', params: { id: data.role_id } });
      }
    } else {
      await roleApi.updateRole(Number(roleId), {
        name: form.value.name,
        state: form.value.state,
        remarks: form.value.remarks || undefined
      });
      message.success('保存成功');
      await loadRole();
    }
  } catch (error: any) {
    message.error(error.message || '保存失败');
  } finally {
    saving.value = false;
  }
};

// 处理对话框中的资源选择（过滤掉已分配的）
const handleDialogResourceSelection = (keys: Array<string | number>) => {
  const assignedResourceIds = new Set(assignedResources.value.map(resource => resource.id).filter((id): id is number => id !== undefined));
  // 只保留未分配的资源ID
  dialogSelectedResourceIds.value = keys.filter(key => {
    const id = Number(key);
    return !assignedResourceIds.has(id);
  });
};

// 添加资源
const handleAddResource = async () => {
  if (dialogSelectedResourceIds.value.length === 0) {
    message.warning('请至少选择一个未分配的权限');
    return false;
  }
  
  try {
    for (const resourceId of dialogSelectedResourceIds.value) {
      await roleApi.assignResourceToRole(Number(roleId), Number(resourceId));
    }
    message.success('分配成功');
    showAddResourceDialog.value = false;
    dialogSelectedResourceIds.value = [];
    await loadRoleResources();
    return true;
  } catch (error: any) {
    message.error(error.message || '分配失败');
    return false;
  }
};

// 移除资源
const handleRemoveResource = async (resourceId: number) => {
  dialog.warning({
    title: '确认移除',
    content: '确定要从该角色中移除该权限吗？',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await roleApi.removeResourceFromRole(Number(roleId), resourceId);
        message.success('移除成功');
        await loadRoleResources();
      } catch (error: any) {
        message.error(error.message || '移除失败');
      }
    }
  });
};

onMounted(async () => {
  await loadTenants();
  if (!isNew.value) {
    await loadRole();
  }
});
</script>

<style scoped>
.detail-container {
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--n-divider-color);
}

.content-wrapper {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.info-card {
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  border-radius: 8px;
}

.info-card :deep(.n-card-header) {
  padding: 12px 16px;
  border-bottom: 1px solid var(--n-divider-color);
  background: var(--n-color);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.info-card :deep(.n-card-body) {
  padding: 16px;
}
</style>
