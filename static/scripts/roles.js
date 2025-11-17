/** @format */

let roles = [];
let permissions = [];
let allRoles = [];

const MODULE_NAMES = {
	users: "Users",
	roles: "Roles",
	permissions: "Permissions",
	jobs: "Jobs",
	rolls: "Rolls",
	downtimes: "Downtimes",
	scraps: "Scraps",
	ink_usages: "Ink Usages",
	solvent_usages: "Solvent Usages",
	shifts: "Shifts",
	colours: "Colours",
	solvent_types: "Solvent Types",
	scrap_types: "Scrap Types",
	downtime_reasons: "Downtime Reasons",
	flag_reasons: "Flag Reasons",
};

const PERMISSION_LABELS = {
	can_create: "Create",
	can_read: "Read",
	can_update: "Update",
	can_delete: "Delete",
};

document.addEventListener("DOMContentLoaded", () => {
	loadRolesAndPermissions();
	setupEventListeners();
});

function setupEventListeners() {
	document.getElementById("add-role-btn").addEventListener("click", showCreateRoleModal);
	document.getElementById("manage-permissions-btn").addEventListener("click", showPermissionsModal);
	document.getElementById("close-modal").addEventListener("click", hideRoleModal);
	document.getElementById("cancel-btn").addEventListener("click", hideRoleModal);
	document.getElementById("close-permissions-modal").addEventListener("click", hidePermissionsModal);
	document.getElementById("role-form").addEventListener("submit", handleRoleSubmit);
}

async function loadRolesAndPermissions() {
	showLoading(true, "roles-table");
	try {
		const [rolesResponse, permissionsResponse] = await Promise.all([
			fetch("/api/roles").then(handleApiResponse),
			fetch("/api/permissions").then(handleApiResponse),
		]);

		roles = rolesResponse;
		permissions = permissionsResponse;
		allRoles = [...roles];

		updateStats();
		renderRolesTable(roles);
		renderPermissionsMatrix();
		showLoading(false, "roles-table");
	} catch (error) {
		showNotification(error.message, "error");
		showLoading(false, "roles-table");
	}
}

function updateStats() {
	document.getElementById("total-roles").textContent = roles.length;
	document.getElementById("total-permissions").textContent = permissions.length;

	const adminRoles = roles.filter(
		(role) => role.name.toLowerCase().includes("admin") || role.description.toLowerCase().includes("full access")
	).length;
	document.getElementById("admin-roles").textContent = adminRoles;

	const readonlyRoles = roles.filter((role) => role.description.toLowerCase().includes("read-only")).length;
	document.getElementById("readonly-roles").textContent = readonlyRoles;
}

function renderRolesTable(rolesToRender) {
	const tbody = document.getElementById("roles-table-body");
	const table = document.getElementById("roles-table");

	if (rolesToRender.length === 0) {
		tbody.innerHTML = '<tr><td colspan="5" class="text-center py-8 text-gray-500">No roles found</td></tr>';
		table.style.display = "table";
		return;
	}

	tbody.innerHTML = rolesToRender
		.map((role) => {
			const rolePermissions = permissions.filter((p) => p.role_id === role.id);
			return `
            <tr>
                <td class="font-mono text-sm text-gray-600">${role.id}</td>
                <td><div class="font-semibold text-gray-800">${escapeHtml(role.name)}</div></td>
                <td class="text-gray-600">${escapeHtml(role.description)}</td>
                <td><span class="role-badge ${getRoleBadgeClass(role)}">${rolePermissions.length} modules</span></td>
                <td>
                    <div class="flex gap-2">
                        <button class="btn btn-primary edit-btn" onclick="editRole(${role.id})">
                            <i class="fas fa-edit"></i> Edit
                        </button>
                        ${
							role.id !== 1
								? `<button class="btn btn-danger delete-btn" onclick="deleteRole(${role.id})"><i class="fas fa-trash"></i> Delete</button>`
								: ""
						}
                    </div>
                </td>
            </tr>
        `;
		})
		.join("");

	table.style.display = "table";
}

function getRoleBadgeClass(role) {
	if (role.name.toLowerCase().includes("admin")) return "role-admin";
	if (role.description.toLowerCase().includes("read-only")) return "role-user";
	return "role-moderator";
}

function renderPermissionsMatrix() {
	const thead = document.querySelector("#permissions-table thead tr");
	const tbody = document.getElementById("permissions-table-body");

	thead.innerHTML = "<th>Module</th>";
	tbody.innerHTML = "";

	roles.forEach((role) => {
		const th = document.createElement("th");
		th.className = "role-header";
		th.textContent = role.name;
		thead.appendChild(th);
	});

	const modules = [...new Set(permissions.map((p) => p.model))];

	modules.forEach((module) => {
		const row = document.createElement("tr");
		const moduleName = MODULE_NAMES[module] || module;

		const moduleCell = document.createElement("td");
		moduleCell.textContent = moduleName;
		moduleCell.className = "font-semibold";
		row.appendChild(moduleCell);

		roles.forEach((role) => {
			const rolePermissions = permissions.filter((p) => p.role_id === role.id && p.model === module);
			const cell = document.createElement("td");
			cell.className = "permission-cell";

			if (rolePermissions.length > 0) {
				const perm = rolePermissions[0];
				const hasAll = perm.can_create && perm.can_read && perm.can_update && perm.can_delete;
				const hasSome = perm.can_create || perm.can_read || perm.can_update || perm.can_delete;

				let badgeClass = "permission-no";
				let badgeText = "No Access";

				if (hasAll) {
					badgeClass = "permission-yes";
					badgeText = "Full Access";
				} else if (hasSome) {
					badgeClass = "permission-partial";
					badgeText = "Partial Access";
				}

				cell.innerHTML = `<span class="permission-badge ${badgeClass}">${badgeText}</span>`;
			} else {
				cell.innerHTML = '<span class="permission-badge permission-no">No Access</span>';
			}
			row.appendChild(cell);
		});
		tbody.appendChild(row);
	});
}

function showCreateRoleModal() {
	document.getElementById("modal-title").textContent = "Create New Role";
	document.getElementById("role-form").reset();
	document.getElementById("submit-btn").textContent = "Create Role";
	document.getElementById("role-form").removeAttribute("data-edit-id");
	document.getElementById("role-modal").style.display = "flex";
}

function hideRoleModal() {
	document.getElementById("role-modal").style.display = "none";
}

function showPermissionsModal() {
	document.getElementById("permissions-modal-title").textContent = "Manage Permissions";
	renderPermissionsManagement();
	document.getElementById("permissions-modal").style.display = "flex";
}

function hidePermissionsModal() {
	document.getElementById("permissions-modal").style.display = "none";
}

function renderPermissionsManagement() {
	const content = document.getElementById("permissions-content");
	content.innerHTML = `
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div><h4 class="font-semibold">Role Permissions Configuration</h4><p class="text-sm text-gray-600">Configure access rights for each role</p></div>
                <select id="role-selector" class="form-input w-64">${roles
					.map((role) => `<option value="${role.id}">${role.name}</option>`)
					.join("")}</select>
            </div>
            <div id="permission-controls">${renderPermissionControls(roles[0].id)}</div>
            <div class="flex justify-end gap-3 pt-4 border-t">
                <button type="button" class="btn btn-secondary" onclick="hidePermissionsModal()">Cancel</button>
                <button type="button" class="btn btn-primary" onclick="savePermissions()">Save Permissions</button>
            </div>
        </div>
    `;

	document.getElementById("role-selector").addEventListener("change", (e) => {
		document.getElementById("permission-controls").innerHTML = renderPermissionControls(parseInt(e.target.value));
	});
}

function renderPermissionControls(roleId) {
	const rolePermissions = permissions.filter((p) => p.role_id === roleId);
	return `
        <div class="permission-grid">
            ${Object.entries(MODULE_NAMES)
				.map(([key, name]) => {
					const modulePerms = rolePermissions.find((p) => p.model === key) || {
						can_create: false,
						can_read: false,
						can_update: false,
						can_delete: false,
					};
					return `
                    <div class="permission-group">
                        <h4>${name}</h4>
                        <div class="permission-checkboxes">
                            ${Object.entries(PERMISSION_LABELS)
								.map(
									([permKey, label]) => `
                                <div class="checkbox-item">
                                    <input type="checkbox" id="perm-${roleId}-${key}-${permKey}" 
                                           ${modulePerms[permKey] ? "checked" : ""}
                                           data-role="${roleId}" data-module="${key}" data-permission="${permKey}">
                                    <label for="perm-${roleId}-${key}-${permKey}">${label}</label>
                                </div>
                            `
								)
								.join("")}
                        </div>
                    </div>
                `;
				})
				.join("")}
        </div>
    `;
}

async function handleRoleSubmit(e) {
	e.preventDefault();
	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const formData = {
		name: document.getElementById("role-name").value,
		description: document.getElementById("role-description").value,
	};

	const editId = document.getElementById("role-form").getAttribute("data-edit-id");
	const isEdit = !!editId;

	try {
		let result;
		if (isEdit) {
			formData.id = parseInt(editId);
			result = await updateRole(formData);
			const index = roles.findIndex((r) => r.id === result.id);
			if (index !== -1) {
				roles[index] = result;
			}
		} else {
			result = await createRole(formData);
			roles.unshift(result);
		}

		showNotification(`Role ${isEdit ? "updated" : "created"} successfully!`, "success");
		hideRoleModal();
		updateStats();
		renderRolesTable(roles);
		renderPermissionsMatrix();
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(submitBtn, false);
	}
}

async function createRole(roleData) {
	const response = await fetch("/api/roles/create", {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(roleData),
	});
	return await handleApiResponse(response);
}

async function updateRole(roleData) {
	const response = await fetch("/api/roles/update", {
		method: "PUT",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(roleData),
	});
	return await handleApiResponse(response);
}

async function savePermissions() {
	const roleId = parseInt(document.getElementById("role-selector").value);
	const checkboxes = document.querySelectorAll('input[type="checkbox"][data-role]');

	const MODEL_TO_CONTENT_TYPE = {
		users: 1,
		roles: 2,
		permissions: 3,
		jobs: 4,
		rolls: 5,
		downtimes: 6,
		scraps: 7,
		ink_usages: 8,
		solvent_usages: 9,
		shifts: 10,
		colours: 11,
		solvent_types: 12,
		scrap_types: 13,
		downtime_reasons: 14,
		flag_reasons: 15,
	};

	const permissionsToSave = [];
	const processedModules = new Set();

	checkboxes.forEach((checkbox) => {
		if (parseInt(checkbox.dataset.role) === roleId && checkbox.checked) {
			processedModules.add(checkbox.dataset.module);
		}
	});

	processedModules.forEach((module) => {
		const content_type_id = MODEL_TO_CONTENT_TYPE[module];
		if (!content_type_id) return;

		const permData = {
			role_id: roleId,
			content_type_id: content_type_id,
			can_create: document.getElementById(`perm-${roleId}-${module}-can_create`)?.checked || false,
			can_read: document.getElementById(`perm-${roleId}-${module}-can_read`)?.checked || false,
			can_update: document.getElementById(`perm-${roleId}-${module}-can_update`)?.checked || false,
			can_delete: document.getElementById(`perm-${roleId}-${module}-can_delete`)?.checked || false,
		};

		const existing = permissions.find((p) => p.role_id === roleId && p.model === module);
		if (existing) {
			permData.id = existing.id;
		}

		permissionsToSave.push(permData);
	});

	try {
		const saveResults = await Promise.all(
			permissionsToSave.map(async (permData) => {
				const url = permData.id ? "/api/permissions/update" : "/api/permissions/create";
				const method = permData.id ? "PUT" : "POST";

				const response = await fetch(url, {
					method: method,
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify(permData),
				});
				return await handleApiResponse(response);
			})
		);

		saveResults.forEach((result, index) => {
			const existingIndex = permissions.findIndex((p) => p.id === result.id);
			if (existingIndex !== -1) {
				permissions[existingIndex] = result;
			} else {
				permissions.push(result);
			}
		});

		showNotification("Permissions saved successfully!", "success");
		hidePermissionsModal();
		updateStats();
		renderRolesTable(roles);
		renderPermissionsMatrix();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function editRole(roleId) {
	const role = roles.find((r) => r.id === roleId);
	if (!role) return;

	document.getElementById("role-name").value = role.name;
	document.getElementById("role-description").value = role.description;
	document.getElementById("modal-title").textContent = "Edit Role";
	document.getElementById("submit-btn").textContent = "Update Role";
	document.getElementById("role-form").setAttribute("data-edit-id", roleId);
	document.getElementById("role-modal").style.display = "flex";
}

async function deleteRole(roleId) {
	if (!confirm("Are you sure you want to delete this role? This action cannot be undone.")) return;

	const deleteBtn = document.querySelector(`.delete-btn[onclick="deleteRole(${roleId})"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch("/api/roles/delete", {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: roleId }),
		});
		await handleApiResponse(response);

		roles = roles.filter((r) => r.id !== roleId);
		permissions = permissions.filter((p) => p.role_id !== roleId);

		showNotification("Role deleted successfully!", "success");
		updateStats();
		renderRolesTable(roles);
		renderPermissionsMatrix();
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		if (deleteBtn) setButtonLoading(deleteBtn, false);
	}
}
