/**
 * Role type definitions — mirror the Rust `Role` struct in
 * `src-tauri/src/commands/role.rs`. The Rust side is authoritative; if you
 * add a field there, add it here too.
 *
 * `RoleApi` keeps the wire format (snake_case) so it round-trips through
 * Tauri's `invoke` boundary without manual mapping. UI code consumes the
 * normalised `Role` type via `toRole()`.
 */

export interface RoleApi {
  id: number;
  name: string;
  display_name: string;
  description: string;
  icon: string;
  is_system: boolean;
}

export interface Role {
  id: number;
  name: string;
  displayName: string;
  description: string;
  icon: string;
  isSystem: boolean;
}

export function toRole(api: RoleApi): Role {
  return {
    id: api.id,
    name: api.name,
    displayName: api.display_name,
    description: api.description,
    icon: api.icon,
    isSystem: api.is_system,
  };
}

export interface UserRoleSelection {
  /** Selected role ids (multi-select supported per FR5). */
  roleIds: number[];
  /** Persisted to localStorage; `null` means "not yet chosen". */
  selectedAt: number | null;
}
