/** @format */
let machines = [];
let currentPage = 1;
let itemsPerPage = 10;
let totalCount = 0;
let sections = [];

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	await loadSections();
	await loadMachines();
	setupEventListeners();
}

function showLoading(show) {
	const loadingMessage = document.getElementById("machines-loading-message");
	const table = document.getElementById("machines-table");

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
	try {
		const response = await fetch("/api/sections");
		const result = await handleApiResponse(response);
		sections = result;
		populateSectionFilters();
	} catch (error) {
		console.error("Failed to load sections:", error);
	}
}

async function loadMachines() {
	showLoading(true);
	try {
		const params = new URLSearchParams();
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/machines/filter?${params}`);
		const result = await handleApiResponse(response);

		machines = result.data;
		totalCount = result.total_count;
		updateMachineStats();
		renderMachines();
		renderPagination();
		updatePerPageOptions(totalCount);
	} catch (error) {
		document.getElementById("machines-table-body").innerHTML =
			'<tr><td colspan="5" class="text-center text-red-500 py-4">Failed to load machines</td></tr>';
		showNotification(error.message, "error");
	} finally {
		showLoading(false);
	}
}

function updateMachineStats() {
	const totalMachines = totalCount;
	const totalJobs = machines.reduce((sum, machine) => sum + machine.job_count, 0);
	const activeSections = new Set(machines.map((m) => m.section_id)).size;

	document.getElementById("total-machines").textContent = totalMachines;
	document.getElementById("machines-total-jobs").textContent = totalJobs;
	document.getElementById("active-sections").textContent = activeSections;
}

async function applyFilters() {
	const applyBtn = document.getElementById("machines-apply-filter");
	setButtonLoading(applyBtn, true);
	showLoading(true);

	try {
		const params = new URLSearchParams();
		const nameFilter = document.getElementById("filter-machine-name").value;
		const labelFilter = document.getElementById("filter-machine-label").value;
		const sectionFilter = document.getElementById("filter-machine-section").value;
		const hasJobs = document.getElementById("filter-has-jobs").value;

		if (nameFilter) params.append("name", nameFilter);
		if (labelFilter) params.append("label", labelFilter);
		if (sectionFilter) params.append("section_id", sectionFilter);
		if (hasJobs === "yes") params.append("has_jobs", "true");
		if (hasJobs === "no") params.append("has_jobs", "false");

		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/machines/filter?${params}`);
		const result = await handleApiResponse(response);

		machines = result.data;
		totalCount = result.total_count;
		updateMachineStats();
		renderMachines();
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
	document.getElementById("filter-machine-name").value = "";
	document.getElementById("filter-machine-label").value = "";
	document.getElementById("filter-machine-section").value = "";
	document.getElementById("filter-has-jobs").value = "";

	currentPage = 1;
	applyFilters();
}

function populateSectionFilters() {
	const filterSelect = document.getElementById("filter-machine-section");
	const modalSelect = document.getElementById("machine-section");

	let filterOptions = '<option value="">All Sections</option>';
	let modalOptions = '<option value="">Select Section</option>';

	sections.forEach((section) => {
		filterOptions += `<option value="${section.id}">${escapeHtml(section.name)}</option>`;
		modalOptions += `<option value="${section.id}">${escapeHtml(section.name)}</option>`;
	});

	filterSelect.innerHTML = filterOptions;
	modalSelect.innerHTML = modalOptions;
}

function renderMachines() {
	const tbody = document.getElementById("machines-table-body");

	if (machines.length === 0) {
		tbody.innerHTML = '<tr><td colspan="5" class="text-center text-gray-500 py-4">No machines found</td></tr>';
		return;
	}

	tbody.innerHTML = "";

	machines.forEach((machine) => {
		const row = document.createElement("tr");
		row.className = "hover:bg-gray-50";

		row.innerHTML = `
			<td class="py-3 px-4 font-medium">${escapeHtml(machine.name)}</td>
			<td class="py-3 px-4">${escapeHtml(machine.label)}</td>
			<td class="py-3 px-4">${escapeHtml(machine.section_name)}</td>
			<td class="py-3 px-4 text-center">${machine.job_count}</td>
			<td class="py-3 px-4">
				<div class="flex gap-2 justify-center">
					<button class="text-blue-600 hover:text-blue-800 edit-btn" data-id="${machine.id}">
						<i class="fas fa-edit"></i>
					</button>
					<button class="text-red-600 hover:text-red-800 delete-btn" data-id="${machine.id}">
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
			editMachine(id);
		});
	});

	document.querySelectorAll(".delete-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			const id = this.dataset.id;
			deleteMachine(id);
		});
	});
}

function renderPagination() {
	const totalPages = Math.ceil(totalCount / itemsPerPage);
	const paginationContainer = document.getElementById("machines-pagination");

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

function openMachineModal(machineId = null) {
	const modal = document.getElementById("machine-modal");
	const title = document.getElementById("machine-modal-title");
	const form = document.getElementById("machine-form");

	if (machineId) {
		title.textContent = "Edit Machine";
		const machine = machines.find((m) => m.id === parseInt(machineId));
		if (machine) {
			populateMachineForm(machine);
		}
	} else {
		title.textContent = "Add Machine";
		form.reset();
		document.getElementById("machine-id").value = "";
	}

	modal.style.display = "flex";
}

function closeMachineModal() {
	document.getElementById("machine-modal").style.display = "none";
}

function populateMachineForm(machine) {
	document.getElementById("machine-id").value = machine.id;
	document.getElementById("machine-name").value = machine.name;
	document.getElementById("machine-label").value = machine.label;
	document.getElementById("machine-section").value = machine.section_id;
}

function handleMachineFormSubmit(e) {
	e.preventDefault();

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const machineId = document.getElementById("machine-id").value;
	const formData = {
		name: document.getElementById("machine-name").value,
		label: document.getElementById("machine-label").value,
		section_id: parseInt(document.getElementById("machine-section").value),
	};

	if (machineId) {
		formData.id = parseInt(machineId);
		updateMachine(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	} else {
		createMachine(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	}
}

async function createMachine(machineData) {
	try {
		const response = await fetch("/api/machines/create", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(machineData),
		});
		await handleApiResponse(response);

		showNotification("Machine created successfully!", "success");
		closeMachineModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function updateMachine(machineData) {
	try {
		const response = await fetch("/api/machines/update", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(machineData),
		});
		await handleApiResponse(response);

		showNotification("Machine updated successfully!", "success");
		closeMachineModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function deleteMachine(machineId) {
	if (!confirm("Are you sure you want to delete this machine?")) return;

	const deleteBtn = document.querySelector(`.delete-btn[data-id="${machineId}"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch("/api/machines/delete", {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: parseInt(machineId) }),
		});
		await handleApiResponse(response);

		showNotification("Machine deleted successfully!", "success");
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		if (deleteBtn) setButtonLoading(deleteBtn, false);
	}
}

function editMachine(machineId) {
	openMachineModal(machineId);
}

async function exportToExcel() {
	const exportBtn = document.getElementById("machines-export-btn");
	setButtonLoading(exportBtn, true);

	try {
		const params = new URLSearchParams();
		const nameFilter = document.getElementById("filter-machine-name").value;
		const labelFilter = document.getElementById("filter-machine-label").value;
		const sectionFilter = document.getElementById("filter-machine-section").value;
		const hasJobs = document.getElementById("filter-has-jobs").value;

		if (nameFilter) params.append("name", nameFilter);
		if (labelFilter) params.append("label", labelFilter);
		if (sectionFilter) params.append("section_id", sectionFilter);
		if (hasJobs === "yes") params.append("has_jobs", "true");
		if (hasJobs === "no") params.append("has_jobs", "false");

		const response = await fetch(`/api/machines/filter?${params}`);
		const result = await handleApiResponse(response);
		const filteredMachines = result.data;

		if (filteredMachines.length === 0) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filteredMachines.map((machine) => {
			return {
				Name: machine.name,
				Label: machine.label,
				Section: machine.section_name,
				"Total Jobs": machine.job_count,
			};
		});

		const worksheet = XLSX.utils.json_to_sheet(data);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, worksheet, "Machines");
		const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
		saveAsExcel(excelBuffer, "machines.xlsx");

		showNotification("Machines exported successfully!", "success");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(exportBtn, false);
	}
}

function setupEventListeners() {
	document.getElementById("machines-apply-filter").addEventListener("click", applyFilters);
	document.getElementById("machines-clear-filter").addEventListener("click", clearFilters);
	document.getElementById("add-machine-btn").addEventListener("click", () => openMachineModal());
	document.getElementById("close-machine-modal").addEventListener("click", closeMachineModal);
	document.getElementById("cancel-machine-btn").addEventListener("click", closeMachineModal);
	document.getElementById("machine-form").addEventListener("submit", handleMachineFormSubmit);

	const exportBtn = document.getElementById("machines-export-btn");
	if (exportBtn) {
		exportBtn.addEventListener("click", exportToExcel);
	}

	document.getElementById("per-page").addEventListener("change", function () {
		itemsPerPage = parseInt(this.value);
		currentPage = 1;
		applyFilters();
	});
}
