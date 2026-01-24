/** @format */

let searchTerm = "";
let users = [];
let roles = [];
let sections = [];
let filteredUsers = [];

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	await loadFilterOptions();
	await loadUsers();
	setupEventListeners();
}

async function loadFilterOptions() {
	try {
		const [rolesResponse, sectionsResponse] = await Promise.all([
			fetch("/api/roles").then(handleApiResponse),
			fetch("/api/sections").then(handleApiResponse),
		]);
		roles = rolesResponse;
		sections = sectionsResponse;
		populateRoleDropdown();
		populateSectionFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function populateSectionFilters() {
	const sectionFilter = document.getElementById("filter-section");
	sectionFilter.innerHTML = '<option value="">All Sections</option>';
	sections.forEach((section) => {
		const option = document.createElement("option");
		option.value = section.id;
		option.textContent = section.name;
		sectionFilter.appendChild(option);
	});
}

function applyFilters() {
	const applyBtn = document.getElementById("apply-filter");
	setButtonLoading(applyBtn, true);

	const roleFilter = document.getElementById("filter-role").value;
	const statusFilter = document.getElementById("filter-status").value;
	const sectionFilter = document.getElementById("filter-section").value;

	filteredUsers = users.filter((user) => {
		if (roleFilter && user.role_id !== parseInt(roleFilter)) return false;
		if (statusFilter && user.status !== statusFilter) return false;
		if (sectionFilter && !user.section_ids.includes(parseInt(sectionFilter))) return false;
		return true;
	});

	updateUserStats(filteredUsers);
	renderUsers();
	setButtonLoading(applyBtn, false);
}

function clearFilters() {
	document.getElementById("filter-role").value = "";
	document.getElementById("filter-status").value = "";
	document.getElementById("filter-section").value = "";
	filteredUsers = [...users];
	updateUserStats(filteredUsers);
	renderUsers();
}

function searchUsers(term) {
	searchTerm = term.toLowerCase();
	if (!searchTerm) {
		filteredUsers = [...users];
	} else {
		filteredUsers = users.filter(
			(user) =>
				user.staffid.toLowerCase().includes(searchTerm) ||
				(user.phone_number && user.phone_number.toLowerCase().includes(searchTerm)) ||
				user.full_name.toLowerCase().includes(searchTerm) ||
				(roles.find((r) => r.id === user.role_id)?.name || "").toLowerCase().includes(searchTerm),
		);
	}
	updateUserStats(filteredUsers);
	renderUsers();
}

async function loadUsers() {
	showLoading(true, "users-table");
	try {
		const response = await fetch("/api/users");
		users = await handleApiResponse(response);
		filteredUsers = [...users];
		updateUserStats(users);
		renderUsers();
	} catch (error) {
		document.getElementById("users-table-body").innerHTML =
			'<tr><td colspan="7" class="text-center text-red-500 py-4">Failed to load users</td></tr>';
		showNotification(error.message, "error");
	} finally {
		showLoading(false, "users-table");
	}
}

function updateUserStats(users) {
	const totalUsers = users.length;
	const activeUsers = users.filter((user) => user.status === "active").length;
	const inactiveUsers = totalUsers - activeUsers;
	const sectionUsers = users.filter((user) => user.section_ids.length > 0).length;
	document.getElementById("total-users").textContent = totalUsers;
	document.getElementById("active-users").textContent = activeUsers;
	document.getElementById("inactive-users").textContent = inactiveUsers;
	document.getElementById("section-users").textContent = sectionUsers;
}

function renderUsers() {
	const tbody = document.getElementById("users-table-body");

	if (filteredUsers.length === 0) {
		tbody.innerHTML = '<tr><td colspan="7" class="text-center text-gray-500 py-4">No users found</td></tr>';
		return;
	}

	tbody.innerHTML = "";
	filteredUsers.forEach((user) => {
		const row = document.createElement("tr");
		row.className = "hover:bg-gray-50";
		row.innerHTML = `
            <td class="py-3 px-4 font-medium">${escapeHtml(user.staffid)}</td>
            <td class="py-3 px-4">${escapeHtml(user.full_name)}</td>
            <td class="py-3 px-4">${escapeHtml(roles.find((r) => r.id === user.role_id)?.name || "Unknown")}</td>
            <td class="py-3 px-4">${escapeHtml(user.phone_number || "N/A")}</td>
            <td class="py-3 px-4">
                <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
					user.status === "active" ? "bg-green-100 text-green-800" : "bg-red-100 text-red-800"
				}">
                    ${escapeHtml(user.status)}
                </span>
            </td>
            <td class="py-3 px-4 text-center">${user.section_ids.length}</td>
            <td class="py-3 px-4">
                <div class="flex gap-2 justify-center">
                    <button class="text-blue-600 hover:text-blue-800 edit-btn" data-id="${user.id}">
                        <i class="fas fa-edit"></i>
                    </button>
                    <button class="text-red-600 hover:text-red-800 delete-btn" data-id="${user.id}">
                        <i class="fas fa-trash"></i>
                    </button>
                </div>
            </td>
        `;
		tbody.appendChild(row);
	});

	document.querySelectorAll(".edit-btn").forEach((btn) => {
		btn.addEventListener("click", () => editUser(btn.dataset.id));
	});

	document.querySelectorAll(".delete-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			deleteUser(this.dataset.id);
		});
	});
}

async function openModal(userId = null) {
	const modal = document.getElementById("user-modal");
	const title = document.getElementById("modal-title");
	const form = document.getElementById("user-form");

	if (userId) {
		title.textContent = "Edit User";
		const user = users.find((u) => u.id === parseInt(userId));
		if (user) {
			populateForm(user);
		}
	} else {
		title.textContent = "Add User";
		form.reset();
		document.getElementById("user-id").value = "";
		document.getElementById("password").value = "";
		populateSectionCheckboxes([]);
	}
	modal.style.display = "flex";
}

function closeModal() {
	document.getElementById("user-modal").style.display = "none";
}

function populateForm(user) {
	document.getElementById("user-id").value = user.id;
	document.getElementById("full-name").value = user.full_name;
	document.getElementById("staff-id").value = user.staffid;
	document.getElementById("phone-number").value = user.phone_number || "";
	document.getElementById("status").value = user.status;
	document.getElementById("role").value = user.role_id;
	document.getElementById("password").value = "";
	populateSectionCheckboxes(user.section_ids);
}

function populateSectionCheckboxes(sectionIds) {
	const sectionsContainer = document.getElementById("sections-checkboxes");
	sectionsContainer.innerHTML = "";
	sectionsContainer.className = "flex flex-wrap gap-4";

	sections.forEach((section) => {
		const checkboxDiv = document.createElement("div");
		checkboxDiv.className = "flex items-center gap-2 min-w-[120px]";
		const isChecked = sectionIds.includes(section.id);
		checkboxDiv.innerHTML = `
            <input type="checkbox" id="section-${section.id}" value="${section.id}" 
                   ${isChecked ? "checked" : ""} class="rounded border-gray-300 text-blue-600 focus:ring-blue-500">
            <label for="section-${section.id}" class="text-sm text-gray-700 whitespace-nowrap">${escapeHtml(section.name)}</label>
        `;
		sectionsContainer.appendChild(checkboxDiv);
	});
}

function populateRoleDropdown() {
	const roleSelect = document.getElementById("role");
	const filterRole = document.getElementById("filter-role");

	roleSelect.innerHTML = '<option value="">Select a role</option>';
	filterRole.innerHTML = '<option value="">All Roles</option>';

	roles.forEach((role) => {
		const option = document.createElement("option");
		option.value = role.id;
		option.textContent = role.name;
		roleSelect.appendChild(option);

		const filterOption = document.createElement("option");
		filterOption.value = role.id;
		filterOption.textContent = role.name;
		filterRole.appendChild(filterOption);
	});
}

function getSelectedSections() {
	const checkboxes = document.querySelectorAll('#sections-checkboxes input[type="checkbox"]:checked');
	return Array.from(checkboxes).map((checkbox) => parseInt(checkbox.value));
}

async function handleFormSubmit(e) {
	e.preventDefault();
	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const userId = document.getElementById("user-id").value;
	const formData = {
		full_name: document.getElementById("full-name").value,
		staffid: document.getElementById("staff-id").value,
		phone_number: document.getElementById("phone-number").value,
		status: document.getElementById("status").value,
		role_id: parseInt(document.getElementById("role").value),
		section_ids: getSelectedSections(),
	};

	const password = document.getElementById("password").value;
	if (password) {
		formData.password = password;
	}

	try {
		if (userId) {
			formData.id = parseInt(userId);
			const updatedUser = await updateUser(formData);
			const index = users.findIndex((u) => u.id === updatedUser.id);
			if (index !== -1) {
				users[index] = updatedUser;
			}
		} else {
			const newUser = await createUser(formData);
			users.unshift(newUser);
		}
		showNotification("User saved successfully!", "success");
		closeModal();
		filteredUsers = [...users];
		updateUserStats(users);
		renderUsers();
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(submitBtn, false);
	}
}

async function createUser(userData) {
	const response = await fetch("/api/users/create", {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(userData),
	});
	return await handleApiResponse(response);
}

async function updateUser(userData) {
	const response = await fetch("/api/users/update", {
		method: "PUT",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(userData),
	});
	return await handleApiResponse(response);
}

function editUser(userId) {
	openModal(userId);
}

async function deleteUser(userId) {
	if (!confirm("Are you sure you want to delete this user?")) return;

	const deleteBtn = document.querySelector(`.delete-btn[data-id="${userId}"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch("/api/users/delete", {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: parseInt(userId) }),
		});
		await handleApiResponse(response);
		showNotification("User deleted successfully!", "success");
		users = users.filter((u) => u.id !== parseInt(userId));
		filteredUsers = filteredUsers.filter((u) => u.id !== parseInt(userId));
		updateUserStats(users);
		renderUsers();
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		if (deleteBtn) setButtonLoading(deleteBtn, false);
	}
}

async function exportToExcel() {
	const exportBtn = document.getElementById("export-btn");
	setButtonLoading(exportBtn, true);

	try {
		if (filteredUsers.length === 0) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filteredUsers.map((user) => {
			return {
				"Staff ID": user.staffid,
				"Full Name": user.full_name,
				"Phone Number": user.phone_number || "N/A",
				Status: user.status,
				Role: roles.find((r) => r.id === user.role_id)?.name || "Unknown",
				"Assigned Sections": user.section_ids.length,
				"Created At": formatDate(user.created_at),
			};
		});

		const worksheet = XLSX.utils.json_to_sheet(data);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, worksheet, "Users");
		const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
		saveAsExcel(excelBuffer, "users_export.xlsx");

		showNotification("Users exported successfully!", "success");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(exportBtn, false);
	}
}

function setupEventListeners() {
	document.getElementById("apply-filter").addEventListener("click", applyFilters);
	document.getElementById("clear-filter").addEventListener("click", clearFilters);
	document.getElementById("add-user-btn").addEventListener("click", () => openModal());
	document.getElementById("close-modal").addEventListener("click", closeModal);
	document.getElementById("cancel-btn").addEventListener("click", closeModal);
	document.getElementById("user-form").addEventListener("submit", handleFormSubmit);
	document.getElementById("search-users").addEventListener("input", function (e) {
		searchUsers(e.target.value);
	});

	const exportBtn = document.getElementById("export-btn");
	if (exportBtn) {
		exportBtn.addEventListener("click", exportToExcel);
	}
}
