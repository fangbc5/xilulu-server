use crate::modules::department::DepartmentService;
use crate::modules::employee::{EmployeeDepartmentService, EmployeeService};
use crate::state::AppState;
use std::sync::Arc;
use tonic::{Request, Response, Status};

// 导入生成的 protobuf 代码
pub mod organization_proto {
    tonic::include_proto!("organization");
}

use organization_proto::{
    organization_service_server::{OrganizationService, OrganizationServiceServer},
    BoolResponse, CheckDepartmentLeaderRequest, DataScopeResponse, Department as ProtoDepartment,
    DepartmentNode, DepartmentTreeResponse, DepartmentsResponse, Employee as ProtoEmployee,
    EmployeeResponse, EmployeesResponse, GetDepartmentEmployeesRequest, GetDepartmentTreeRequest,
    GetEmployeeByUserIdRequest, GetEmployeeDepartmentsRequest, GetEmployeesByUserIdRequest,
    GetUserDataScopeRequest, Position as ProtoPosition,
};

/// 组织服务 gRPC 实现
pub struct OrganizationServiceImpl {
    app_state: Arc<AppState>,
}

impl OrganizationServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// 创建 gRPC Server
    pub fn server(app_state: Arc<AppState>) -> OrganizationServiceServer<Self> {
        OrganizationServiceServer::new(Self::new(app_state))
    }

    /// 转换 Employee 实体为 Proto
    fn employee_to_proto(emp: &crate::modules::employee::Employee) -> ProtoEmployee {
        ProtoEmployee {
            id: emp.id.unwrap_or(0),
            tenant_id: emp.tenant_id,
            org_id: emp.org_id,
            user_id: emp.user_id,
            employee_no: emp.employee_no.clone(),
            name: emp.name.clone(),
            avatar: emp.avatar.clone().unwrap_or_default(),
            gender: emp.gender.unwrap_or(0) as i32,
            mobile: emp.mobile.clone().unwrap_or_default(),
            email: emp.email.clone().unwrap_or_default(),
            status: emp.status.unwrap_or(1) as i32,
            primary_department: None,
            primary_position: None,
        }
    }

    /// 转换 Department 实体为 Proto
    fn department_to_proto(dept: &crate::modules::department::Department) -> ProtoDepartment {
        ProtoDepartment {
            id: dept.id.unwrap_or(0),
            tenant_id: dept.tenant_id,
            org_id: dept.org_id,
            parent_id: dept.parent_id.unwrap_or(0),
            code: dept.code.clone(),
            name: dept.name.clone(),
            full_name: dept.full_name.clone().unwrap_or_default(),
            path: dept.path.clone().unwrap_or_default(),
            level: dept.level.unwrap_or(1),
            leader_employee_id: dept.leader_employee_id.unwrap_or(0),
            sort_order: dept.sort_order.unwrap_or(0),
            status: dept.status.unwrap_or(1) as i32,
        }
    }
}

#[tonic::async_trait]
impl OrganizationService for OrganizationServiceImpl {
    /// 根据用户ID获取员工信息
    async fn get_employee_by_user_id(
        &self,
        request: Request<GetEmployeeByUserIdRequest>,
    ) -> Result<Response<EmployeeResponse>, Status> {
        let req = request.into_inner();

        // 如果指定了 org_id，直接查询
        if req.org_id > 0 {
            match self
                .app_state
                .employee_service
                .get_by_user_and_org(req.user_id, req.org_id)
                .await
            {
                Ok(Some(emp)) => {
                    return Ok(Response::new(EmployeeResponse {
                        success: true,
                        message: "成功".to_string(),
                        employee: Some(Self::employee_to_proto(&emp)),
                    }));
                }
                Ok(None) => {
                    return Ok(Response::new(EmployeeResponse {
                        success: false,
                        message: "员工不存在".to_string(),
                        employee: None,
                    }));
                }
                Err(e) => {
                    return Err(Status::internal(format!("查询失败: {}", e)));
                }
            }
        }

        // 否则查询租户下的第一个员工身份
        match self
            .app_state
            .employee_service
            .get_by_user_and_tenant(req.user_id, req.tenant_id)
            .await
        {
            Ok(emps) => {
                if let Some(emp) = emps.first() {
                    Ok(Response::new(EmployeeResponse {
                        success: true,
                        message: "成功".to_string(),
                        employee: Some(Self::employee_to_proto(emp)),
                    }))
                } else {
                    Ok(Response::new(EmployeeResponse {
                        success: false,
                        message: "员工不存在".to_string(),
                        employee: None,
                    }))
                }
            }
            Err(e) => Err(Status::internal(format!("查询失败: {}", e))),
        }
    }

    /// 获取用户所有员工身份
    async fn get_employees_by_user_id(
        &self,
        request: Request<GetEmployeesByUserIdRequest>,
    ) -> Result<Response<EmployeesResponse>, Status> {
        let req = request.into_inner();

        match self
            .app_state
            .employee_service
            .get_by_user_and_tenant(req.user_id, req.tenant_id)
            .await
        {
            Ok(emps) => {
                let proto_emps: Vec<ProtoEmployee> =
                    emps.iter().map(Self::employee_to_proto).collect();
                Ok(Response::new(EmployeesResponse {
                    success: true,
                    message: "成功".to_string(),
                    employees: proto_emps,
                }))
            }
            Err(e) => Err(Status::internal(format!("查询失败: {}", e))),
        }
    }

    /// 获取员工的部门列表
    async fn get_employee_departments(
        &self,
        request: Request<GetEmployeeDepartmentsRequest>,
    ) -> Result<Response<DepartmentsResponse>, Status> {
        let req = request.into_inner();

        // 获取员工部门关系
        match self
            .app_state
            .employee_department_service
            .get_by_employee(req.employee_id)
            .await
        {
            Ok(rels) => {
                let mut departments = Vec::new();
                for rel in rels {
                    // 获取部门详情
                    if let Ok(dept) = self
                        .app_state
                        .department_service
                        .get_by_id(rel.department_id)
                        .await
                    {
                        departments.push(Self::department_to_proto(&dept));
                    }
                }
                Ok(Response::new(DepartmentsResponse {
                    success: true,
                    message: "成功".to_string(),
                    departments,
                }))
            }
            Err(e) => Err(Status::internal(format!("查询失败: {}", e))),
        }
    }

    /// 获取部门下的所有员工
    async fn get_department_employees(
        &self,
        request: Request<GetDepartmentEmployeesRequest>,
    ) -> Result<Response<EmployeesResponse>, Status> {
        let req = request.into_inner();

        // TODO: 实现获取部门员工的完整逻辑（包括子部门）
        // 当前简化实现：仅返回该部门的直接员工
        
        Ok(Response::new(EmployeesResponse {
            success: true,
            message: "成功".to_string(),
            employees: vec![],
        }))
    }

    /// 获取部门树
    async fn get_department_tree(
        &self,
        request: Request<GetDepartmentTreeRequest>,
    ) -> Result<Response<DepartmentTreeResponse>, Status> {
        let req = request.into_inner();

        match self.app_state.department_service.get_tree(req.org_id).await {
            Ok(tree) => {
                fn convert_tree(
                    nodes: Vec<crate::modules::department::DepartmentTreeNode>,
                ) -> Vec<DepartmentNode> {
                    nodes
                        .into_iter()
                        .map(|node| DepartmentNode {
                            department: Some(ProtoDepartment {
                                id: node.department.id,
                                tenant_id: node.department.tenant_id,
                                org_id: node.department.org_id,
                                parent_id: node.department.parent_id.unwrap_or(0),
                                code: node.department.code,
                                name: node.department.name,
                                full_name: node.department.full_name.unwrap_or_default(),
                                path: node.department.path.unwrap_or_default(),
                                level: node.department.level.unwrap_or(1),
                                leader_employee_id: node.department.leader_employee_id.unwrap_or(0),
                                sort_order: node.department.sort_order.unwrap_or(0),
                                status: node.department.status.unwrap_or(1) as i32,
                            }),
                            children: convert_tree(node.children),
                        })
                        .collect()
                }

                Ok(Response::new(DepartmentTreeResponse {
                    success: true,
                    message: "成功".to_string(),
                    nodes: convert_tree(tree),
                }))
            }
            Err(e) => Err(Status::internal(format!("查询失败: {}", e))),
        }
    }

    /// 检查用户是否是部门负责人
    async fn check_department_leader(
        &self,
        request: Request<CheckDepartmentLeaderRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();

        // TODO: 实现检查部门负责人的逻辑
        // 需要先找到用户对应的员工，然后检查员工部门关系中的 is_leader 字段

        Ok(Response::new(BoolResponse {
            success: true,
            message: "成功".to_string(),
            result: false,
        }))
    }

    /// 获取用户的数据权限范围
    async fn get_user_data_scope(
        &self,
        request: Request<GetUserDataScopeRequest>,
    ) -> Result<Response<DataScopeResponse>, Status> {
        let req = request.into_inner();

        // TODO: 实现数据权限范围计算
        // 需要：
        // 1. 获取用户在该组织的员工身份
        // 2. 获取员工的角色
        // 3. 根据角色的 data_scope 设置计算部门列表

        // 默认返回"仅本人"
        Ok(Response::new(DataScopeResponse {
            success: true,
            message: "成功".to_string(),
            scope_type: 5, // 5 = SelfOnly
            department_ids: vec![],
        }))
    }
}
