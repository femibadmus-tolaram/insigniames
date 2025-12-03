/** @format */
let sections = [];
let orderTypes = [];
let currentPage = 1;
let itemsPerPage = 10;
let totalCount = 0;
let currentSectionId = null;

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	setupEventListeners();
	await loadSections();
	await loadOrderTypes();
}

function showLoading(show) {
	const loadingMessage = document.getElementById("sections-loading-message");
	const table = document.getElementById("sections-table");

	if (show) {
		loadingMessage.style.display = "block";
		table.style.display = "none";
	} else {
		loadingMessage.style.display = "none";
		table.style.display = "table";
	}
}

function setButtonLoading(button, loading) {
	if (loading) {
		button.disabled = true;
		const originalHTML = button.innerHTML;
		button.setAttribute("data-original-html", originalHTML);
		button.innerHTML = '<i class="fas fa-spinner fa-spin"></i> Loading...';
	} else {
		button.disabled = false;
		const originalHTML = button.getAttribute("data-original-html");
		if (originalHTML) {
			button.innerHTML = originalHTML;
		}
	}
}

async function loadSections() {
	showLoading(true);
	try {
		const params = new URLSearchParams();
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/sections/filter?${params}`);
		const result = await handleApiResponse(response);

		sections = result.data;
		totalCount = result.total_count;
		updateSectionStats();
		renderSections();
		renderPagination();
		updatePerPageOptions(totalCount);
	} catch (error) {
		document.getElementById("sections-table-body").innerHTML =
			'<tr><td colspan="5" class="text-center text-red-500 py-4">Failed to load sections</td></tr>';
		showNotification(error.message, "error");
	} finally {
		showLoading(false);
	}
}

async function loadOrderTypes() {
	try {
		const response = await fetch("/api/lookups/manufacturing-order-types");
		const result = await handleApiResponse(response);
		orderTypes = result;
	} catch (error) {
		console.error("Failed to load order types:", error);
	}
}

function updateSectionStats() {
	const totalSections = totalCount;
	const totalMachines = sections.reduce((sum, section) => sum + (section.machine_count || 0), 0);
	const totalOrderTypeCount = sections.reduce((sum, section) => sum + (section.order_type_ids.length || 0), 0);
	const totalJobs = sections.reduce((sum, section) => sum + (section.job_count || 0), 0);

	document.getElementById("total-sections").textContent = totalSections;
	document.getElementById("sections-total-machines").textContent = totalMachines;
	document.getElementById("total-order-types").textContent = totalOrderTypeCount;
	document.getElementById("sections-total-jobs").textContent = totalJobs;
}

async function applyFilters() {
	const applyBtn = document.getElementById("sections-apply-filter");
	setButtonLoading(applyBtn, true);
	showLoading(true);

	try {
		const params = new URLSearchParams();
		const nameFilter = document.getElementById("filter-section-name").value;
		const hasMachines = document.getElementById("filter-has-machines").value;
		const hasOrderTypes = document.getElementById("filter-has-order-types").value;

		if (nameFilter) params.append("name", nameFilter);
		if (hasMachines === "yes") params.append("has_machines", "true");
		if (hasMachines === "no") params.append("has_machines", "false");
		if (hasOrderTypes === "yes") params.append("has_order_types", "true");
		if (hasOrderTypes === "no") params.append("has_order_types", "false");

		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/sections/filter?${params}`);
		const result = await handleApiResponse(response);

		sections = result.data;
		totalCount = result.total_count;
		updateSectionStats();
		renderSections();
		renderPagination();
		updatePerPageOptions(totalCount);
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(applyBtn, false);
		showLoading(false);
	}
}

function clearFilters() {
	document.getElementById("filter-section-name").value = "";
	document.getElementById("filter-has-machines").value = "";
	document.getElementById("filter-has-order-types").value = "";

	currentPage = 1;
	applyFilters();
}

function renderSections() {
	const tbody = document.getElementById("sections-table-body");

	if (sections.length === 0) {
		tbody.innerHTML = '<tr><td colspan="5" class="text-center text-gray-500 py-4">No sections found</td></tr>';
		return;
	}

	tbody.innerHTML = "";

	sections.forEach((section) => {
		const row = document.createElement("tr");
		row.className = "hover:bg-gray-50";

		const orderTypeCount = section.order_type_ids ? section.order_type_ids.length : 0;

		row.innerHTML = `
			<td class="py-3 px-4 font-medium">${escapeHtml(section.name)}</td>
			<td class="py-3 px-4 text-center">${section.machine_count || 0}</td>
			<td class="py-3 px-4 text-center">${orderTypeCount}</td>
			<td class="py-3 px-4 text-center">${section.job_count || 0}</td>
			<td class="py-3 px-4">
				<div class="flex gap-2 justify-center">
					<button class="text-blue-600 hover:text-blue-800 edit-btn" data-id="${section.id}">
						<i class="fas fa-edit"></i>
					</button>
					<button class="text-green-600 hover:text-green-800 order-types-btn" data-id="${section.id}" data-name="${escapeHtml(section.name)}">
						<i class="fas fa-clipboard-list"></i>
					</button>
					<button class="text-red-600 hover:text-red-800 delete-btn" data-id="${section.id}">
						<i class="fas fa-trash"></i>
					</button>
				</div>
			</td>
		`;

		tbody.appendChild(row);
	});

	document.querySelectorAll(".edit-btn").forEach((btn) => {
		btn.addEventListener("click", () => {
			const id = btn.dataset.id;
			editSection(id);
		});
	});

	document.querySelectorAll(".order-types-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			const id = this.dataset.id;
			const name = this.dataset.name;
			manageOrderTypes(id, name);
		});
	});

	document.querySelectorAll(".delete-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			const id = this.dataset.id;
			deleteSection(id);
		});
	});
}

function renderPagination() {
	const totalPages = Math.ceil(totalCount / itemsPerPage);
	const paginationContainer = document.getElementById("sections-pagination");

	if (totalPages <= 1) {
		paginationContainer.innerHTML = "";
		return;
	}

	const startItem = (currentPage - 1) * itemsPerPage + 1;
	const endItem = Math.min(currentPage * itemsPerPage, totalCount);

	let paginationHTML = `
		<div class="flex items-center gap-4">
			<div class="text-sm text-gray-600">
				Showing ${startItem} to ${endItem} of ${totalCount} entries
			</div>
			<div class="flex gap-1">
				<button class="pagination-btn" ${currentPage === 1 ? "disabled" : ""} id="prev-page">
					<i class="fas fa-chevron-left"></i>
				</button>
				<button class="pagination-btn" ${currentPage === totalPages ? "disabled" : ""} id="next-page">
					<i class="fas fa-chevron-right"></i>
				</button>
			</div>
		</div>
	`;

	paginationContainer.innerHTML = paginationHTML;

	document.getElementById("prev-page")?.addEventListener("click", () => {
		if (currentPage > 1) {
			currentPage--;
			applyFilters();
		}
	});

	document.getElementById("next-page")?.addEventListener("click", () => {
		if (currentPage < totalPages) {
			currentPage++;
			applyFilters();
		}
	});
}

function openSectionModal(sectionId = null) {
	const modal = document.getElementById("section-modal");
	const title = document.getElementById("section-modal-title");
	const form = document.getElementById("section-form");

	if (sectionId) {
		title.textContent = "Edit Section";
		const section = sections.find((s) => s.id === parseInt(sectionId));
		if (section) {
			populateSectionForm(section);
		}
	} else {
		title.textContent = "Add Section";
		form.reset();
		document.getElementById("section-id").value = "";
	}

	modal.style.display = "flex";
}

function closeSectionModal() {
	document.getElementById("section-modal").style.display = "none";
}

function openOrderTypesModal() {
	document.getElementById("order-types-modal").style.display = "flex";
}

function closeOrderTypesModal() {
	document.getElementById("order-types-modal").style.display = "none";
}

function populateSectionForm(section) {
	document.getElementById("section-id").value = section.id;
	document.getElementById("section-name").value = section.name;
}

function manageOrderTypes(sectionId, sectionName) {
	currentSectionId = sectionId;
	const modalTitle = document.getElementById("order-types-modal-title");
	modalTitle.textContent = `Manage Order Types - ${sectionName}`;

	const section = sections.find((s) => s.id === parseInt(sectionId));
	if (!section) return;

	const orderTypesList = document.getElementById("order-types-list");
	orderTypesList.innerHTML = "";

	const sectionOrderTypeIds = section.order_type_ids || [];

	orderTypes.forEach((orderType) => {
		const isChecked = sectionOrderTypeIds.includes(orderType.id);
		const checkbox = document.createElement("div");
		checkbox.className = "flex items-center";
		checkbox.innerHTML = `
			<input type="checkbox" id="order-type-${orderType.id}" value="${orderType.id}" ${isChecked ? "checked" : ""} class="mr-2 h-4 w-4">
			<label for="order-type-${orderType.id}" class="text-sm text-gray-700">${escapeHtml(orderType.name)}</label>
		`;
		orderTypesList.appendChild(checkbox);
	});

	openOrderTypesModal();
}

function handleSectionFormSubmit(e) {
	e.preventDefault();

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const sectionId = document.getElementById("section-id").value;
	const formData = {
		name: document.getElementById("section-name").value,
	};

	if (sectionId) {
		formData.id = parseInt(sectionId);
		updateSection(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	} else {
		createSection(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	}
}

async function saveOrderTypes() {
	const saveBtn = document.getElementById("save-order-types-btn");
	setButtonLoading(saveBtn, true);

	try {
		const checkboxes = document.querySelectorAll('#order-types-list input[type="checkbox"]');
		const selectedOrderTypes = Array.from(checkboxes)
			.filter((cb) => cb.checked)
			.map((cb) => parseInt(cb.value));

		const formData = {
			id: parseInt(currentSectionId),
			order_type_ids: selectedOrderTypes,
		};

		const response = await fetch("/api/sections/update", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(formData),
		});
		await handleApiResponse(response);

		showNotification("Order types updated successfully!", "success");
		closeOrderTypesModal();
		await loadSections();
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(saveBtn, false);
	}
}

async function createSection(sectionData) {
	try {
		const response = await fetch("/api/sections/create", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(sectionData),
		});
		await handleApiResponse(response);

		showNotification("Section created successfully!", "success");
		closeSectionModal();
		await loadSections();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function updateSection(sectionData) {
	try {
		const response = await fetch("/api/sections/update", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(sectionData),
		});
		await handleApiResponse(response);

		showNotification("Section updated successfully!", "success");
		closeSectionModal();
		await loadSections();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function deleteSection(sectionId) {
	if (!confirm("Are you sure you want to delete this section?")) return;

	const deleteBtn = document.querySelector(`.delete-btn[data-id="${sectionId}"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch("/api/sections/delete", {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: parseInt(sectionId) }),
		});
		await handleApiResponse(response);

		showNotification("Section deleted successfully!", "success");
		await loadSections();
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		if (deleteBtn) setButtonLoading(deleteBtn, false);
	}
}

function editSection(sectionId) {
	openSectionModal(sectionId);
}

async function exportToExcel() {
	const exportBtn = document.getElementById("sections-export-btn");
	setButtonLoading(exportBtn, true);

	try {
		const params = new URLSearchParams();
		const nameFilter = document.getElementById("filter-section-name").value;
		const hasMachines = document.getElementById("filter-has-machines").value;
		const hasOrderTypes = document.getElementById("filter-has-order-types").value;

		if (nameFilter) params.append("name", nameFilter);
		if (hasMachines === "yes") params.append("has_machines", "true");
		if (hasMachines === "no") params.append("has_machines", "false");
		if (hasOrderTypes === "yes") params.append("has_order_types", "true");
		if (hasOrderTypes === "no") params.append("has_order_types", "false");

		const response = await fetch(`/api/sections/filter?${params}`);
		const result = await handleApiResponse(response);
		const filteredSections = result.data;

		if (filteredSections.length === 0) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filteredSections.map((section) => {
			const orderTypeCount = section.order_type_ids ? section.order_type_ids.length : 0;
			return {
				Name: section.name,
				Machines: section.machine_count || 0,
				"Order Types": orderTypeCount,
				"Total Jobs": section.job_count || 0,
			};
		});

		const worksheet = XLSX.utils.json_to_sheet(data);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, worksheet, "Sections");
		const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
		saveAsExcel(excelBuffer, "sections.xlsx");

		showNotification("Sections exported successfully!", "success");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(exportBtn, false);
	}
}

function setupEventListeners() {
	document.getElementById("sections-apply-filter").addEventListener("click", applyFilters);
	document.getElementById("sections-clear-filter").addEventListener("click", clearFilters);
	document.getElementById("add-section-btn").addEventListener("click", () => openSectionModal());
	document.getElementById("close-section-modal").addEventListener("click", closeSectionModal);
	document.getElementById("cancel-section-btn").addEventListener("click", closeSectionModal);
	document.getElementById("section-form").addEventListener("submit", handleSectionFormSubmit);

	document.getElementById("close-order-types-modal").addEventListener("click", closeOrderTypesModal);
	document.getElementById("cancel-order-types-btn").addEventListener("click", closeOrderTypesModal);
	document.getElementById("save-order-types-btn").addEventListener("click", saveOrderTypes);

	const exportBtn = document.getElementById("sections-export-btn");
	if (exportBtn) {
		exportBtn.addEventListener("click", exportToExcel);
	}

	document.getElementById("per-page").addEventListener("change", function () {
		itemsPerPage = parseInt(this.value);
		currentPage = 1;
		applyFilters();
	});
}
