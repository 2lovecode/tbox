/**
 * 角色类型定义
 *
 * 对应后端 Rust 结构体: src-tauri/src/commands/role.rs
 */

export interface Role {
  id: number;
  name: string;
  display_name: string;
  description: string;
  icon: string;
  is_system: boolean;
}
