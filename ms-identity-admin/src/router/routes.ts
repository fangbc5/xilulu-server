import { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/pages/auth/Login.vue'),
    meta: { requiresAuth: false }
  },
  {
    path: '/',
    component: () => import('@/layout/index.vue'),
    redirect: '/dashboard',
    meta: { requiresAuth: true },
    children: [
      {
        path: 'dashboard',
        name: 'Dashboard',
        component: () => import('@/pages/Dashboard.vue'),
        meta: { title: '仪表盘', requiresAuth: true }
      },
      {
        path: 'users',
        name: 'Users',
        component: () => import('@/pages/user/UserList.vue'),
        meta: { title: '用户管理', requiresAuth: true }
      },
      {
        path: 'users/:id',
        name: 'UserDetail',
        component: () => import('@/pages/user/UserDetail.vue'),
        meta: { title: '用户详情', requiresAuth: true }
      },
      {
        path: 'tenants',
        name: 'Tenants',
        component: () => import('@/pages/tenant/TenantList.vue'),
        meta: { title: '租户管理', requiresAuth: true }
      },
      {
        path: 'tenants/:id',
        name: 'TenantDetail',
        component: () => import('@/pages/tenant/TenantDetail.vue'),
        meta: { title: '租户详情', requiresAuth: true }
      },
      {
        path: 'tenants/:tenantId/users/:userId/roles',
        name: 'TenantUserRoles',
        component: () => import('@/pages/tenant/TenantUserRoles.vue'),
        meta: { title: '用户角色管理', requiresAuth: true }
      },
      {
        path: 'roles',
        name: 'Roles',
        component: () => import('@/pages/auth/RoleList.vue'),
        meta: { title: '角色管理', requiresAuth: true }
      },
      {
        path: 'roles/:id',
        name: 'RoleDetail',
        component: () => import('@/pages/auth/RoleDetail.vue'),
        meta: { title: '角色详情', requiresAuth: true }
      },
      {
        path: 'resources',
        name: 'Resources',
        component: () => import('@/pages/resource/ResourceList.vue'),
        meta: { title: '资源管理', requiresAuth: true }
      },
      {
        path: 'resources/:id',
        name: 'ResourceDetail',
        component: () => import('@/pages/resource/ResourceDetail.vue'),
        meta: { title: '资源详情', requiresAuth: true }
      },
      {
        path: 'applications',
        name: 'Applications',
        component: () => import('@/pages/application/ApplicationList.vue'),
        meta: { title: '应用管理', requiresAuth: true }
      },
      {
        path: 'applications/:id',
        name: 'ApplicationDetail',
        component: () => import('@/pages/application/ApplicationDetail.vue'),
        meta: { title: '应用详情', requiresAuth: true }
      },
      {
        path: 'plans',
        name: 'Plans',
        component: () => import('@/pages/plan/PlanList.vue'),
        meta: { title: '套餐管理', requiresAuth: true }
      },
      {
        path: 'plans/:id',
        name: 'PlanDetail',
        component: () => import('@/pages/plan/PlanDetail.vue'),
        meta: { title: '套餐详情', requiresAuth: true }
      },
      {
        path: 'settings',
        name: 'Settings',
        component: () => import('@/pages/Settings.vue'),
        meta: { title: '系统设置', requiresAuth: true }
      }
    ]
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    component: () => import('@/pages/friendly_error/NotFound.vue')
  }
];

export default routes;

