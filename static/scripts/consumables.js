/** @format */
let consumables = [];
let colours = [];
let solventTypes = [];
let users = [];
let currentPage = 1;
let itemsPerPage = 10;
let totalCount = 0;
let shifts = [
	{ id: 1, name: "Day" },
	{ id: 2, name: "Night" },
];

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	await loadFilterOptions();
	await loadConsumables();
	setupEventListeners();
}

async function loadFilterOptions() {
	try {
		const [coloursResponse, solventTypesResponse, usersResponse] = await Promise.all([
			fetch("/api/lookups/colours").then(handleApiResponse),
			fetch("/api/lookups/solvent-types").then(handleApiResponse),
			fetch("/api/users").then(handleApiResponse),
		]);

		colours = coloursResponse;
		solventTypes = solventTypesResponse;
		users = usersResponse;

		populateSelect(
			"filter-type",
			[
				{ id: "ink", name: "Ink" },
				{ id: "solvent", name: "Solvent" },
			],
			"name",
			"All Types"
		);
		populateSelect("filter-user", users, "full_name", "All Users");
		populateSelect("ink-colour", colours, "name", "Select Colour");
		populateSelect("solvent-type", solventTypes, "name", "Select Solvent Type");
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function loadConsumables() {
	showLoading(true, "ink-table");
	showLoading(true, "solvent-table");
	try {
		const params = new URLSearchParams();
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const [inkResponse, solventResponse] = await Promise.all([
			fetch(`/api/ink-usages/filter?${params}`).then(handleApiResponse),
			fetch(`/api/solvent-usages/filter?${params}`).then(handleApiResponse),
		]);

		const inkData = inkResponse.data || inkResponse;
		const solventData = solventResponse.data || solventResponse;

		consumables = [...inkData.map((item) => ({ ...item, type: "ink" })), ...solventData.map((item) => ({ ...item, type: "solvent" }))];
		totalCount = (inkResponse.total_count || inkData.length) + (solventResponse.total_count || solventData.length);

		updateConsumableStats(consumables);
		renderConsumables(consumables);
		renderPagination();
		updatePerPageOptions(totalCount);
	} catch (error) {
		document.getElementById("ink-table-body").innerHTML =
			'<tr><td colspan="7" class="text-center text-red-500 py-4">Failed to load ink records</td></tr>';
		document.getElementById("solvent-table-body").innerHTML =
			'<tr><td colspan="6" class="text-center text-red-500 py-4">Failed to load solvent records</td></tr>';
		showNotification(error.message, "error");
	} finally {
		showLoading(false, "ink-table");
		showLoading(false, "solvent-table");
	}
}

function updateConsumableStats(consumables) {
	const totalRecords = consumables.length;
	const totalInk = consumables.filter((c) => c.type === "ink").reduce((sum, c) => sum + (c.kgs_issued || 0), 0);
	const totalSolvent = consumables.filter((c) => c.type === "solvent").reduce((sum, c) => sum + (c.kgs_issued || 0), 0);
	const totalWeight = totalInk + totalSolvent;

	document.getElementById("total-records").textContent = totalRecords;
	document.getElementById("total-ink").textContent = `${formatWeight(totalInk)}`;
	document.getElementById("total-solvent").textContent = `${formatWeight(totalSolvent)}`;
	document.getElementById("total-weight").textContent = `${formatWeight(totalWeight)}`;
}

function renderConsumables(consumablesToRender) {
	const inkTbody = document.getElementById("ink-table-body");
	const solventTbody = document.getElementById("solvent-table-body");

	const inkData = consumablesToRender.filter((c) => c.type === "ink");
	const solventData = consumablesToRender.filter((c) => c.type === "solvent");

	if (!inkData || inkData.length === 0) {
		inkTbody.innerHTML = '<tr><td colspan="7" class="text-center text-gray-500 py-4">No ink records found</td></tr>';
	} else {
		inkTbody.innerHTML = "";
		inkData.forEach((consumable) => {
			const shift = shifts.find((s) => s.id === consumable.shift_id);
			const createdBy = users.find((u) => u.id === consumable.created_by);
			const colour = colours.find((c) => c.id === consumable.colour_id);

			const row = document.createElement("tr");
			row.className = "hover:bg-gray-50";

			row.innerHTML = `
				<td class="py-3 px-4">${escapeHtml(shift?.name || "Unknown")}</td>
				<td class="py-3 px-4">${escapeHtml(colour?.name || "Unknown")}</td>
				<td class="py-3 px-4">${escapeHtml(consumable.batch_code || "N/A")}</td>
				<td class="py-3 px-4 text-center">${(consumable.kgs_issued || 0).toFixed(2)}</td>
				<td class="py-3 px-4">${escapeHtml(createdBy?.full_name || "System")}</td>
				<td class="py-3 px-4">${formatDate(consumable.created_at)}</td>
				<td class="py-3 px-4">
					<div class="flex gap-2">
						<button class="text-blue-600 hover:text-blue-800 edit-btn" data-id="${consumable.id}" data-type="ink">
							<i class="fas fa-edit"></i>
						</button>
						<button class="text-red-600 hover:text-red-800 delete-btn" data-id="${consumable.id}" data-type="ink">
							<i class="fas fa-trash"></i>
						</button>
					</div>
				</td>
			`;

			inkTbody.appendChild(row);
		});
	}

	if (!solventData || solventData.length === 0) {
		solventTbody.innerHTML = '<tr><td colspan="6" class="text-center text-gray-500 py-4">No solvent records found</td></tr>';
	} else {
		solventTbody.innerHTML = "";
		solventData.forEach((consumable) => {
			const shift = shifts.find((s) => s.id === consumable.shift_id);
			const createdBy = users.find((u) => u.id === consumable.created_by);
			const solvent = solventTypes.find((s) => s.id === consumable.solvent_type_id);

			const row = document.createElement("tr");
			row.className = "hover:bg-gray-50";

			row.innerHTML = `
				<td class="py-3 px-4">${escapeHtml(shift?.name || "Unknown")}</td>
				<td class="py-3 px-4">${escapeHtml(solvent?.name || "Unknown")}</td>
				<td class="py-3 px-4 text-center">${(consumable.kgs_issued || 0).toFixed(2)}</td>
				<td class="py-3 px-4">${escapeHtml(createdBy?.full_name || "System")}</td>
				<td class="py-3 px-4">${formatDate(consumable.created_at)}</td>
				<td class="py-3 px-4">
					<div class="flex gap-2">
						<button class="text-blue-600 hover:text-blue-800 edit-btn" data-id="${consumable.id}" data-type="solvent">
							<i class="fas fa-edit"></i>
						</button>
						<button class="text-red-600 hover:text-red-800 delete-btn" data-id="${consumable.id}" data-type="solvent">
							<i class="fas fa-trash"></i>
						</button>
					</div>
				</td>
			`;

			solventTbody.appendChild(row);
		});
	}

	document.querySelectorAll(".edit-btn").forEach((btn) => {
		btn.addEventListener("click", () => {
			const type = btn.dataset.type;
			const id = btn.dataset.id;
			if (type === "ink") {
				editInk(id);
			} else {
				editSolvent(id);
			}
		});
	});

	document.querySelectorAll(".delete-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			const type = this.dataset.type;
			const id = this.dataset.id;
			if (type === "ink") {
				deleteInk(id);
			} else {
				deleteSolvent(id);
			}
		});
	});
}

function renderPagination() {
	const totalPages = Math.ceil(totalCount / itemsPerPage);
	const paginationContainer = document.getElementById("pagination");

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

async function applyFilters() {
	const applyBtn = document.getElementById("apply-filter");
	setButtonLoading(applyBtn, true);

	const typeFilter = document.getElementById("filter-type").value;
	if (typeFilter === "ink") {
		showLoading(true, "ink-table");
	} else if (typeFilter === "solvent") {
		showLoading(true, "solvent-table");
	} else {
		showLoading(true, "ink-table");
		showLoading(true, "solvent-table");
	}

	try {
		const params = new URLSearchParams();
		const startDate = document.getElementById("filter-start-date").value;
		const endDate = document.getElementById("filter-end-date").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const typeFilter = document.getElementById("filter-type").value;
		const userFilter = document.getElementById("filter-user").value;

		if (startDate) params.append("start_date", startDate);
		if (endDate) params.append("end_date", endDate);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (typeFilter) params.append("type", typeFilter);
		if (userFilter) params.append("created_by", userFilter);
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		let inkResponse, solventResponse;

		if (!typeFilter || typeFilter === "ink") {
			inkResponse = await fetch(`/api/ink-usages/filter?${params}`).then(handleApiResponse);
		}
		if (!typeFilter || typeFilter === "solvent") {
			solventResponse = await fetch(`/api/solvent-usages/filter?${params}`).then(handleApiResponse);
		}

		const inkData = inkResponse?.data || inkResponse || [];
		const solventData = solventResponse?.data || solventResponse || [];

		consumables = [...inkData.map((item) => ({ ...item, type: "ink" })), ...solventData.map((item) => ({ ...item, type: "solvent" }))];
		totalCount = (inkResponse?.total_count || inkData.length) + (solventResponse?.total_count || solventData.length);

		updateConsumableStats(consumables);
		renderConsumables(consumables);
		renderPagination();
		updatePerPageOptions(totalCount);
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(applyBtn, false);
		const typeFilter = document.getElementById("filter-type").value;
		if (typeFilter === "ink") {
			showLoading(false, "ink-table");
		} else if (typeFilter === "solvent") {
			showLoading(false, "solvent-table");
		} else {
			showLoading(false, "ink-table");
			showLoading(false, "solvent-table");
		}
	}
}

function clearFilters() {
	document.getElementById("filter-start-date").value = "";
	document.getElementById("filter-end-date").value = "";
	document.getElementById("filter-shift").value = "";
	document.getElementById("filter-type").value = "";
	document.getElementById("filter-user").value = "";

	currentPage = 1;
	applyFilters();
}

function openInkModal(inkId = null) {
	const modal = document.getElementById("ink-modal");
	const title = document.getElementById("ink-modal-title");
	const form = document.getElementById("ink-form");

	if (inkId) {
		title.textContent = "Edit Ink Usage";
		const ink = consumables.find((c) => c.id === parseInt(inkId) && c.type === "ink");
		if (ink) {
			populateInkForm(ink);
			const inkTime = new Date(ink.created_at);
			const now = new Date();
			const hoursDiff = (now - inkTime) / (1000 * 60 * 60);

			if (hoursDiff > 24) {
				document.getElementById("ink-shift").disabled = true;
				document.getElementById("ink-colour").disabled = true;
				document.getElementById("ink-batch").disabled = true;
				document.getElementById("ink-quantity").disabled = true;
				document.querySelector('#ink-form button[type="submit"]').disabled = true;
				document.querySelector('#ink-form button[type="submit"]').innerHTML = "Editing Disabled (Over 24 hours)";
			}
		}
	} else {
		title.textContent = "Add Ink Usage";
		form.reset();
		document.getElementById("ink-id").value = "";
		enableInkForm();
	}

	modal.style.display = "flex";
}

function enableInkForm() {
	document.getElementById("ink-shift").disabled = false;
	document.getElementById("ink-colour").disabled = false;
	document.getElementById("ink-batch").disabled = false;
	document.getElementById("ink-quantity").disabled = false;
	const submitBtn = document.querySelector('#ink-form button[type="submit"]');
	submitBtn.disabled = false;
	submitBtn.innerHTML = "Save";
}

function closeInkModal() {
	document.getElementById("ink-modal").style.display = "none";
	enableInkForm();
}

function populateInkForm(ink) {
	document.getElementById("ink-id").value = ink.id;
	document.getElementById("ink-shift").value = ink.shift_id;
	document.getElementById("ink-colour").value = ink.colour_id;
	document.getElementById("ink-batch").value = ink.batch_code || "";
	document.getElementById("ink-quantity").value = ink.kgs_issued || "";
}

function openSolventModal(solventId = null) {
	const modal = document.getElementById("solvent-modal");
	const title = document.getElementById("solvent-modal-title");
	const form = document.getElementById("solvent-form");

	if (solventId) {
		title.textContent = "Edit Solvent Usage";
		const solvent = consumables.find((c) => c.id === parseInt(solventId) && c.type === "solvent");
		if (solvent) {
			populateSolventForm(solvent);
			const solventTime = new Date(solvent.created_at);
			const now = new Date();
			const hoursDiff = (now - solventTime) / (1000 * 60 * 60);

			if (hoursDiff > 24) {
				document.getElementById("solvent-shift").disabled = true;
				document.getElementById("solvent-type").disabled = true;
				document.getElementById("solvent-quantity").disabled = true;
				document.querySelector('#solvent-form button[type="submit"]').disabled = true;
				document.querySelector('#solvent-form button[type="submit"]').innerHTML = "Editing Disabled (Over 24 hours)";
			}
		}
	} else {
		title.textContent = "Add Solvent Usage";
		form.reset();
		document.getElementById("solvent-id").value = "";
		enableSolventForm();
	}

	modal.style.display = "flex";
}

function enableSolventForm() {
	document.getElementById("solvent-shift").disabled = false;
	document.getElementById("solvent-type").disabled = false;
	document.getElementById("solvent-quantity").disabled = false;
	const submitBtn = document.querySelector('#solvent-form button[type="submit"]');
	submitBtn.disabled = false;
	submitBtn.innerHTML = "Save";
}

function closeSolventModal() {
	document.getElementById("solvent-modal").style.display = "none";
	enableSolventForm();
}

function populateSolventForm(solvent) {
	document.getElementById("solvent-id").value = solvent.id;
	document.getElementById("solvent-shift").value = solvent.shift_id;
	document.getElementById("solvent-type").value = solvent.solvent_type_id;
	document.getElementById("solvent-quantity").value = solvent.kgs_issued || "";
}

function handleInkFormSubmit(e) {
	e.preventDefault();

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const inkId = document.getElementById("ink-id").value;
	const formData = {
		shift_id: parseInt(document.getElementById("ink-shift").value),
		colour_id: parseInt(document.getElementById("ink-colour").value),
		batch_code: document.getElementById("ink-batch").value,
		kgs_issued: parseFloat(document.getElementById("ink-quantity").value),
	};

	if (inkId) {
		formData.id = parseInt(inkId);
		updateInk(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	} else {
		createInk(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	}
}

function handleSolventFormSubmit(e) {
	e.preventDefault();

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const solventId = document.getElementById("solvent-id").value;
	const formData = {
		shift_id: parseInt(document.getElementById("solvent-shift").value),
		solvent_type_id: parseInt(document.getElementById("solvent-type").value),
		kgs_issued: parseFloat(document.getElementById("solvent-quantity").value),
	};

	if (solventId) {
		formData.id = parseInt(solventId);
		updateSolvent(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	} else {
		createSolvent(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	}
}

async function createInk(inkData) {
	try {
		const response = await fetch("/api/ink-usages/create", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(inkData),
		});
		await handleApiResponse(response);

		showNotification("Ink usage record created successfully!", "success");
		closeInkModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function updateInk(inkData) {
	try {
		const response = await fetch("/api/ink-usages/update", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(inkData),
		});
		await handleApiResponse(response);

		showNotification("Ink usage record updated successfully!", "success");
		closeInkModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function createSolvent(solventData) {
	try {
		const response = await fetch("/api/solvent-usages/create", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(solventData),
		});
		await handleApiResponse(response);

		showNotification("Solvent usage record created successfully!", "success");
		closeSolventModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function updateSolvent(solventData) {
	try {
		const response = await fetch("/api/solvent-usages/update", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(solventData),
		});
		await handleApiResponse(response);

		showNotification("Solvent usage record updated successfully!", "success");
		closeSolventModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function editInk(inkId) {
	openInkModal(inkId);
}

function editSolvent(solventId) {
	openSolventModal(solventId);
}

async function deleteInk(inkId) {
	if (!confirm("Are you sure you want to delete this ink usage record?")) return;

	const deleteBtn = document.querySelector(`.delete-btn[data-id="${inkId}"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch("/api/ink-usages/delete", {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: parseInt(inkId) }),
		});
		await handleApiResponse(response);

		showNotification("Ink usage record deleted successfully!", "success");
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		if (deleteBtn) setButtonLoading(deleteBtn, false);
	}
}

async function deleteSolvent(solventId) {
	if (!confirm("Are you sure you want to delete this solvent usage record?")) return;

	const deleteBtn = document.querySelector(`.delete-btn[data-id="${solventId}"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch("/api/solvent-usages/delete", {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: parseInt(solventId) }),
		});
		await handleApiResponse(response);

		showNotification("Solvent usage record deleted successfully!", "success");
		await applyFilters();
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
		const params = new URLSearchParams();
		const startDate = document.getElementById("filter-start-date").value;
		const endDate = document.getElementById("filter-end-date").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const typeFilter = document.getElementById("filter-type").value;
		const userFilter = document.getElementById("filter-user").value;

		if (startDate) params.append("start_date", startDate);
		if (endDate) params.append("end_date", endDate);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (typeFilter) params.append("type", typeFilter);
		if (userFilter) params.append("created_by", userFilter);

		let inkResponse, solventResponse;

		if (!typeFilter || typeFilter === "ink") {
			inkResponse = await fetch(`/api/ink-usages/filter?${params}`).then(handleApiResponse);
		}
		if (!typeFilter || typeFilter === "solvent") {
			solventResponse = await fetch(`/api/solvent-usages/filter?${params}`).then(handleApiResponse);
		}

		const inkData = inkResponse?.data || inkResponse || [];
		const solventData = solventResponse?.data || solventResponse || [];
		const filteredConsumables = [...inkData, ...solventData];

		if (filteredConsumables.length === 0) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filteredConsumables.map((consumable) => {
			const shift = shifts.find((s) => s.id === consumable.shift_id);
			const createdBy = users.find((u) => u.id === consumable.created_by);

			let itemName = "Unknown";
			if (consumable.colour_id) {
				const colour = colours.find((c) => c.id === consumable.colour_id);
				itemName = colour?.name || "Unknown";
			} else {
				const solvent = solventTypes.find((s) => s.id === consumable.solvent_type_id);
				itemName = solvent?.name || "Unknown";
			}

			return {
				Type: consumable.colour_id ? "Ink" : "Solvent",
				Shift: shift?.name || "Unknown",
				Item: itemName,
				"Batch Code": consumable.batch_code || "N/A",
				"Quantity (kg)": (consumable.kgs_issued || 0).toFixed(2),
				"Created By": createdBy?.full_name || "System",
				"Created At": formatDate(consumable.created_at),
			};
		});

		const worksheet = XLSX.utils.json_to_sheet(data);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, worksheet, "Consumable Records");
		const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
		saveAsExcel(excelBuffer, "consumable_records.xlsx");

		showNotification("Consumable records exported successfully!", "success");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(exportBtn, false);
	}
}

function setupEventListeners() {
	document.getElementById("apply-filter").addEventListener("click", applyFilters);
	document.getElementById("clear-filter").addEventListener("click", clearFilters);
	document.getElementById("add-ink-btn").addEventListener("click", () => openInkModal());
	document.getElementById("add-solvent-btn").addEventListener("click", () => openSolventModal());
	document.getElementById("close-ink-modal").addEventListener("click", closeInkModal);
	document.getElementById("close-solvent-modal").addEventListener("click", closeSolventModal);
	document.getElementById("cancel-ink-btn").addEventListener("click", closeInkModal);
	document.getElementById("cancel-solvent-btn").addEventListener("click", closeSolventModal);
	document.getElementById("ink-form").addEventListener("submit", handleInkFormSubmit);
	document.getElementById("solvent-form").addEventListener("submit", handleSolventFormSubmit);

	const exportBtn = document.getElementById("export-btn");
	if (exportBtn) {
		exportBtn.addEventListener("click", exportToExcel);
	}

	document.getElementById("per-page").addEventListener("change", function () {
		itemsPerPage = parseInt(this.value);
		currentPage = 1;
		applyFilters();
	});
}
