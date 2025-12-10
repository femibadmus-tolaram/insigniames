/** @format */

let sections = [];
let poCodes = [];
let currentPage = 1;
let itemsPerPage = 10;
let totalCount = 0;
let currentSectionId = null;

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	setupEventListeners();
	await loadPoCodes();
	await loadSections();
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

async function loadPoCodes() {
	try {
		const response = await fetch("/api/lookups/po-codes");
		const result = await handleApiResponse(response);
		poCodes = result;
	} catch (error) {
		console.error("Failed to load PO codes:", error);
	}
}

function updateSectionStats() {
	const totalSections = totalCount;
	const totalMachines = sections.reduce((sum, section) => sum + (section.machine_count || 0), 0);
	const totalPoCodes = sections.reduce((sum, section) => sum + (section.po_code_ids?.length || 0), 0);
	const totalJobs = sections.reduce((sum, section) => sum + (section.job_count || 0), 0);

	document.getElementById("total-sections").textContent = totalSections;
	document.getElementById("sections-total-machines").textContent = totalMachines;
	document.getElementById("total-po-codes").textContent = totalPoCodes;
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
		const hasPoCodes = document.getElementById("filter-po-codes-types").value;

		if (nameFilter) params.append("name", nameFilter);
		if (hasMachines === "yes") params.append("has_machines", "true");
		if (hasMachines === "no") params.append("has_machines", "false");
		if (hasPoCodes === "yes") params.append("has_po_codes", "true");
		if (hasPoCodes === "no") params.append("has_po_codes", "false");

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
	document.getElementById("filter-po-codes-types").value = "";

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

		const poCodeNames =
			section.po_code_ids
				?.map((id) => {
					const poCode = poCodes.find((pc) => pc.id === id);
					return poCode ? poCode.name : "";
				})
				.filter((name) => name !== "") || [];

		row.innerHTML = `
			<td class="py-3 px-4 font-medium">${escapeHtml(section.name)}</td>
			<td class="py-3 px-4 text-center">${section.machine_count || 0}</td>
			<td class="py-3 px-4">
				<div class="text-center">
					${
						poCodeNames.length > 0
							? `<span class="inline-block bg-blue-100 text-blue-800 text-xs px-2 py-1 rounded-full mr-1">${escapeHtml(
									poCodeNames[0]
							  )}</span>` +
							  (poCodeNames.length > 1 ? `<span class="text-gray-500 text-xs">+${poCodeNames.length - 1} more</span>` : "")
							: '<span class="text-gray-400">None</span>'
					}
				</div>
			</td>
			<td class="py-3 px-4 text-center">${section.job_count || 0}</td>
			<td class="py-3 px-4">
				<div class="flex gap-2 justify-center">
					<button class="text-blue-600 hover:text-blue-800 edit-btn" data-id="${section.id}">
						<i class="fas fa-edit"></i>
					</button>
					<button class="text-green-600 hover:text-green-800 po-codes-btn" data-id="${section.id}" data-name="${escapeHtml(section.name)}">
						<i class="fas fa-tags"></i>
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

	document.querySelectorAll(".po-codes-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			const id = this.dataset.id;
			const name = this.dataset.name;
			managePoCodes(id, name);
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
	document.getElementById("po-codes-modal").style.display = "flex";
}

function closeOrderTypesModal() {
	document.getElementById("po-codes-modal").style.display = "none";
}

function populateSectionForm(section) {
	document.getElementById("section-id").value = section.id;
	document.getElementById("section-name").value = section.name;
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

function managePoCodes(sectionId, sectionName) {
	currentSectionId = sectionId;
	const modalTitle = document.getElementById("po-codes-modal-title");
	modalTitle.textContent = `Manage PO Codes - ${sectionName}`;

	const section = sections.find((s) => s.id === parseInt(sectionId));
	if (!section) return;

	const poCodesList = document.getElementById("po-codes-list");
	poCodesList.innerHTML = "";

	poCodes.forEach((poCode) => {
		const checkbox = document.createElement("div");
		checkbox.className = "flex items-center mb-2";
		checkbox.innerHTML = `
			<input type="checkbox" id="po-code-${poCode.id}" name="po_codes" 
				value="${poCode.id}" ${section.po_code_ids?.includes(poCode.id) ? "checked" : ""} 
				class="mr-2 h-4 w-4">
			<label for="po-code-${poCode.id}" class="text-sm text-gray-700">${escapeHtml(poCode.name)}</label>
		`;
		poCodesList.appendChild(checkbox);
	});

	openOrderTypesModal();
}

async function savePoCodes() {
	const saveBtn = document.getElementById("save-po-codes-btn");
	setButtonLoading(saveBtn, true);

	try {
		const selectedCheckboxes = document.querySelectorAll('#po-codes-list input[name="po_codes"]:checked');
		const selectedPoCodes = Array.from(selectedCheckboxes).map((cb) => parseInt(cb.value));

		const formData = {
			id: parseInt(currentSectionId),
			po_code_ids: selectedPoCodes,
		};

		const response = await fetch("/api/sections/update-po-codes", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(formData),
		});
		await handleApiResponse(response);

		const section = sections.find((s) => s.id === parseInt(currentSectionId));
		if (section) {
			section.po_code_ids = selectedPoCodes;
		}

		showNotification("PO codes updated successfully!", "success");
		closeOrderTypesModal();
		renderSections();
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(saveBtn, false);
	}
}

async function exportToExcel() {
	const exportBtn = document.getElementById("sections-export-btn");
	setButtonLoading(exportBtn, true);

	try {
		const params = new URLSearchParams();
		const nameFilter = document.getElementById("filter-section-name").value;
		const hasMachines = document.getElementById("filter-has-machines").value;
		const hasPoCodes = document.getElementById("filter-po-codes-types").value;

		if (nameFilter) params.append("name", nameFilter);
		if (hasMachines === "yes") params.append("has_machines", "true");
		if (hasMachines === "no") params.append("has_machines", "false");
		if (hasPoCodes === "yes") params.append("has_po_codes", "true");
		if (hasPoCodes === "no") params.append("has_po_codes", "false");

		const response = await fetch(`/api/sections/filter?${params}`);
		const result = await handleApiResponse(response);
		const filteredSections = result.data;

		if (filteredSections.length === 0) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filteredSections.map((section) => {
			const poCodeCount = section.po_code_ids ? section.po_code_ids.length : 0;
			return {
				Name: section.name,
				Machines: section.machine_count || 0,
				"PO Codes": poCodeCount,
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

	document.getElementById("close-po-codes-modal").addEventListener("click", closeOrderTypesModal);
	document.getElementById("cancel-po-codes-btn").addEventListener("click", closeOrderTypesModal);
	document.getElementById("save-po-codes-btn").addEventListener("click", savePoCodes);

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

async function updateSection(formData) {
	try {
		const response = await fetch("/api/sections/update", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(formData),
		});
		await handleApiResponse(response);

		showNotification("Section updated successfully!", "success");
		closeSectionModal();
		await loadSections();
	} catch (error) {
		showNotification(error.message, "error");
	}
}
