/** @format */

let inputRolls = [];
let users = [];
let currentPage = 1;
let itemsPerPage = 10;
let showDetails = false;

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	showDetails = localStorage.getItem("inputRollsShowDetails") === "true";
	const toggle = document.getElementById("details-toggle");
	if (toggle) {
		toggle.checked = showDetails;
		toggle.addEventListener("change", toggleDetails);
	}
	await loadUsers();
	await loadInputRolls();
	setupEventListeners();
	setTimeout(() => toggleDetails(), 100);
}

async function loadUsers() {
	try {
		const response = await fetch("/api/users");
		const result = await handleApiResponse(response);
		users = Array.isArray(result) ? result : result.data || [];
	} catch (error) {
		users = [];
	}
}

async function loadInputRolls() {
	showLoading(true, "input-rolls-table");
	try {
		const response = await fetch("/api/input-rolls");
		const result = await handleApiResponse(response);
		inputRolls = Array.isArray(result) ? result : result.data || [];
		applyFilters(false);
	} catch (error) {
		document.getElementById("input-rolls-table-body").innerHTML =
			'<tr><td colspan="9" class="text-center text-red-500 py-4">Failed to load input rolls</td></tr>';
		showNotification(error.message, "error");
	} finally {
		showLoading(false, "input-rolls-table");
	}
}

function applyFilters(resetPage = true) {
	if (resetPage) currentPage = 1;
	const filtered = getFilteredInputRolls();
	const totalCount = filtered.length;
	if (totalCount > 0 && itemsPerPage > totalCount) {
		itemsPerPage = totalCount;
	}
	updateStats(filtered);
	renderInputRolls(filtered);
	renderPagination(totalCount);
	updatePerPageOptions(totalCount);
}

function getFilteredInputRolls() {
	const processOrder = document.getElementById("filter-process-order").value.trim().toLowerCase();
	const batch = document.getElementById("filter-batch").value.trim().toLowerCase();
	const materialDescription = document.getElementById("filter-material-description").value.trim().toLowerCase();
	const createdBy = document.getElementById("filter-created-by").value.trim();
	const status = document.getElementById("filter-status").value;
	const startDate = document.getElementById("filter-start-date").value;
	const endDate = document.getElementById("filter-end-date").value;

	const startDateTime = startDate ? new Date(`${startDate}T00:00:00`) : null;
	const endDateTime = endDate ? new Date(`${endDate}T23:59:59`) : null;

	return inputRolls.filter((roll) => {
		if (processOrder && !String(roll.process_order || "").toLowerCase().includes(processOrder)) return false;
		if (batch && !String(roll.batch || "").toLowerCase().includes(batch)) return false;
		if (materialDescription && !String(roll.material_description || "").toLowerCase().includes(materialDescription)) return false;
		if (createdBy && String(roll.created_by) !== createdBy) return false;

		const consumedWeight = roll.consumed_weight === null || roll.consumed_weight === undefined ? null : Number(roll.consumed_weight);
		const isConsumed = !Number.isNaN(consumedWeight) && consumedWeight > 0;
		if (status === "open" && isConsumed) return false;
		if (status === "consumed" && !isConsumed) return false;

		if (startDateTime || endDateTime) {
			const createdAt = new Date(roll.created_at);
			if (Number.isNaN(createdAt.getTime())) return false;
			if (startDateTime && createdAt < startDateTime) return false;
			if (endDateTime && createdAt > endDateTime) return false;
		}

		return true;
	});
}

function updateStats(filtered) {
	const total = filtered.length;
	const consumed = filtered.filter((roll) => {
		const weight = roll.consumed_weight === null || roll.consumed_weight === undefined ? 0 : Number(roll.consumed_weight);
		return !Number.isNaN(weight) && weight > 0;
	}).length;
	const open = total - consumed;
	const totalConsumedWeight = filtered.reduce((sum, roll) => {
		const weight = roll.consumed_weight === null || roll.consumed_weight === undefined ? 0 : Number(roll.consumed_weight);
		return Number.isNaN(weight) ? sum : sum + weight;
	}, 0);

	document.getElementById("total-input-rolls").textContent = total;
	document.getElementById("open-rolls").textContent = open;
	document.getElementById("consumed-rolls").textContent = consumed;
	document.getElementById("consumed-weight").textContent = totalConsumedWeight ? formatWeight(totalConsumedWeight) : "0";
}

function renderInputRolls(filtered) {
	const tbody = document.getElementById("input-rolls-table-body");
	if (!filtered.length) {
		tbody.innerHTML = '<tr><td colspan="10" class="text-center text-gray-500 py-4">No input rolls found</td></tr>';
		return;
	}

	const startIndex = (currentPage - 1) * itemsPerPage;
	const pageItems = filtered.slice(startIndex, startIndex + itemsPerPage);

	tbody.innerHTML = "";

	pageItems.forEach((roll) => {
		const consumedWeight = roll.consumed_weight === null || roll.consumed_weight === undefined ? null : Number(roll.consumed_weight);
		const isConsumed = !Number.isNaN(consumedWeight) && consumedWeight > 0;
		const statusLabel = isConsumed ? "Consumed" : "Open";
		const statusClass = isConsumed ? "bg-green-100 text-green-800" : "bg-yellow-100 text-yellow-800";
		const createdByName = resolveUserName(roll.created_by);

		const row = document.createElement("tr");
		row.className = "hover:bg-gray-50";
		row.innerHTML = `
			<td class="py-3 px-4">
				<span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${statusClass}">${statusLabel}</span>
			</td>
			<td class="py-3 px-4 font-medium">${escapeHtml(roll.batch || "-")}</td>
			<td class="py-3 px-4">${escapeHtml(roll.process_order || "-")}</td>
			<td class="py-3 px-4">${escapeHtml(roll.material_description || "-")}</td>
			<td class="py-3 px-4 detail-cell" ${!showDetails ? 'style="display: none"' : ""}>${escapeHtml(roll.material_document || "-")}</td>
			<td class="py-3 px-4">${typeof roll.start_meter === "number" ? formatMeter(roll.start_meter) : "-"}</td>
			<td class="py-3 px-4">${escapeHtml(roll.start_weight || "-")}</td>
			<td class="py-3 px-4 detail-cell" ${!showDetails ? 'style="display: none"' : ""}>${consumedWeight === null || Number.isNaN(consumedWeight) ? "-" : formatWeight(consumedWeight)}</td>
			<td class="py-3 px-4">${escapeHtml(createdByName)}</td>
			<td class="py-3 px-4">${formatDateTime(roll.created_at)}</td>
		`;

		tbody.appendChild(row);
	});
}

function toggleDetails() {
	showDetails = !showDetails;
	const toggle = document.getElementById("details-toggle");
	if (toggle) toggle.checked = showDetails;
	const detailColumns = document.querySelectorAll(".details-column");
	const detailCells = document.querySelectorAll(".detail-cell");

	if (showDetails) {
		detailColumns.forEach((col) => (col.style.display = "table-cell"));
		detailCells.forEach((cell) => (cell.style.display = "table-cell"));
	} else {
		detailColumns.forEach((col) => (col.style.display = "none"));
		detailCells.forEach((cell) => (cell.style.display = "none"));
	}

	localStorage.setItem("inputRollsShowDetails", showDetails);
}

function resolveUserName(userId) {
	if (!userId) return "-";
	const user = users.find((u) => u.id === userId);
	if (!user) return String(userId);
	return user.full_name || user.username || user.staffid || String(userId);
}

function renderPagination(totalCount) {
	const paginationContainer = document.getElementById("pagination");
	const totalPages = Math.ceil(totalCount / itemsPerPage);

	if (totalPages <= 1) {
		paginationContainer.innerHTML = "";
		return;
	}

	const startItem = (currentPage - 1) * itemsPerPage + 1;
	const endItem = Math.min(currentPage * itemsPerPage, totalCount);

	paginationContainer.innerHTML = `
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

	document.getElementById("prev-page")?.addEventListener("click", () => {
		if (currentPage > 1) {
			currentPage--;
			applyFilters(false);
		}
	});

	document.getElementById("next-page")?.addEventListener("click", () => {
		if (currentPage < totalPages) {
			currentPage++;
			applyFilters(false);
		}
	});
}

function clearFilters() {
	document.getElementById("filter-process-order").value = "";
	document.getElementById("filter-batch").value = "";
	document.getElementById("filter-material-description").value = "";
	document.getElementById("filter-created-by").value = "";
	document.getElementById("filter-status").value = "";
	document.getElementById("filter-start-date").value = "";
	document.getElementById("filter-end-date").value = "";

	applyFilters();
}

async function exportToExcel() {
	const exportBtn = document.getElementById("export-btn");
	setButtonLoading(exportBtn, true);

	try {
		const filtered = getFilteredInputRolls();
		if (!filtered.length) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filtered.map((roll) => {
			const consumedWeight = roll.consumed_weight === null || roll.consumed_weight === undefined ? null : Number(roll.consumed_weight);
			const isConsumed = !Number.isNaN(consumedWeight) && consumedWeight > 0;

			return {
				Status: isConsumed ? "Consumed" : "Open",
				Batch: roll.batch || "-",
				"Process Order": roll.process_order || "-",
				"Material Description": roll.material_description || "-",
				"Material Document": roll.material_document || "-",
				"Start Meter": typeof roll.start_meter === "number" ? roll.start_meter : "-",
				"Start Weight": roll.start_weight || "-",
				"Consumed Weight": consumedWeight === null || Number.isNaN(consumedWeight) ? "-" : consumedWeight,
				"Created By": resolveUserName(roll.created_by),
				"Created At": formatDateTime(roll.created_at),
			};
		});

		const worksheet = XLSX.utils.json_to_sheet(data);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, worksheet, "Input Rolls");
		const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
		saveAsExcel(excelBuffer, "input_rolls.xlsx");

		showNotification("Input rolls exported successfully!", "success");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(exportBtn, false);
	}
}

function setupEventListeners() {
	document.getElementById("apply-filter").addEventListener("click", () => applyFilters());
	document.getElementById("clear-filter").addEventListener("click", clearFilters);
	document.getElementById("export-btn").addEventListener("click", exportToExcel);
	const perPageSelect = document.getElementById("per-page");
	perPageSelect.addEventListener("change", function () {
		itemsPerPage = parseInt(this.value, 10);
		currentPage = 1;
		applyFilters(false);
	});
}

