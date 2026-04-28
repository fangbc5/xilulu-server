pub mod contacts;
pub mod department;
pub mod employee;
pub mod organization;
pub mod position;

// 重新导出常用类型
pub use department::{Department, DepartmentService};
pub use employee::{Employee, EmployeeDepartment, EmployeePosition, EmployeeService};
pub use organization::{Organization, OrganizationService};
pub use position::{Position, PositionService};
