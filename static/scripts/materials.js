/** @format */
let materials = [];
let currentPage = 1;
let itemsPerPage = 10;
let totalCount = 0;

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	await loadMaterials();
	setupEventListeners();
}

function showLoading(show) {
	const loadingMessage = document.getElementById("materials-loading-message");
	const table = document.getElementById("materials-table");

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

async function loadMaterials() {
	showLoading(true);
	try {
		const params = new URLSearchParams();
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/materials/filter?${params}`);
		const result = await handleApiResponse(response);

		materials = result.data;
		totalCount = result.total_count;
		updateMaterialStats();
		renderMaterials();
		renderPagination();
		updatePerPageOptions(totalCount);
	} catch (error) {
		document.getElementById("materials-table-body").innerHTML =
			'<tr><td colspan="5" class="text-center text-red-500 py-4">Failed to load materials</td></tr>';
		showNotification(error.message, "error");
	} finally {
		showLoading(false);
	}
}

function updateMaterialStats() {
	const totalMaterials = totalCount;
	let withDescriptions = 0;
	let withoutDescriptions = 0;

	materials.forEach((material) => {
		if (material.key === "Loading..." || material.value === "Loading...") {
			withoutDescriptions++;
		} else {
			withDescriptions++;
		}
	});

	document.getElementById("total-materials").textContent = totalMaterials;
	document.getElementById("materials-with-descriptions").textContent = withDescriptions;
	document.getElementById("materials-without-descriptions").textContent = withoutDescriptions;
}

async function applyFilters() {
	const applyBtn = document.getElementById("materials-apply-filter");
	setButtonLoading(applyBtn, true);
	showLoading(true);

	try {
		const params = new URLSearchParams();
		const codeFilter = document.getElementById("filter-material-code").value;
		const keyFilter = document.getElementById("filter-material-key").value;
		const hasDescriptions = document.getElementById("filter-has-descriptions").value;

		if (codeFilter) params.append("code", codeFilter);
		if (keyFilter) params.append("key", keyFilter);
		if (hasDescriptions === "yes") params.append("has_descriptions", "true");
		if (hasDescriptions === "no") params.append("has_descriptions", "false");

		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/materials/filter?${params}`);
		const result = await handleApiResponse(response);

		materials = result.data;
		totalCount = result.total_count;
		updateMaterialStats();
		renderMaterials();
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
	document.getElementById("filter-material-code").value = "";
	document.getElementById("filter-material-key").value = "";
	document.getElementById("filter-has-descriptions").value = "";

	currentPage = 1;
	applyFilters();
}

function renderMaterials() {
	const tbody = document.getElementById("materials-table-body");

	if (materials.length === 0) {
		tbody.innerHTML = '<tr><td colspan="5" class="text-center text-gray-500 py-4">No materials found</td></tr>';
		return;
	}

	tbody.innerHTML = "";

	materials.forEach((material) => {
		const row = document.createElement("tr");
		row.className = "hover:bg-gray-50";

		// Group keys like process_order does
		const keysArray = material.key.split(",").filter((k) => k.trim() !== "");
		const groupedKeys = {};

		keysArray.forEach((key) => {
			if (!groupedKeys[key]) {
				groupedKeys[key] = 1;
			} else {
				groupedKeys[key]++;
			}
		});

		const keysList = Object.entries(groupedKeys)
			.map(([key, count]) => {
				if (count > 1) {
					return `<span class="badge badge-blue mr-1 mb-1" title="${count} items">${escapeHtml(key)} (${count})</span>`;
				}
				return `<span class="badge badge-blue mr-1 mb-1">${escapeHtml(key)}</span>`;
			})
			.join("");

		const materialDetails = material.material_details || {};
		const detailCount = Object.keys(materialDetails).length;
		const hasDetails = detailCount > 0;

		let detailsHTML = "";
		if (hasDetails) {
			detailsHTML = `
				<div class="material-details-container">
					<button class="text-blue-600 hover:text-blue-800 text-sm toggle-details-btn" data-id="${material.id}" data-count="${detailCount}">
						<i class="fas fa-chevron-down mr-1"></i> Show Details (${detailCount})
					</button>
					<div class="details-content hidden mt-2" id="details-${material.id}">
						${Object.entries(materialDetails)
							.map(([key, values]) => {
								const valueArray = values.split(",");
								const uniqueValues = [...new Set(valueArray)];
								return `
								<div class="mb-3 border-l-2 border-blue-200 pl-3">
									<div class="font-medium text-gray-700 mb-1">${escapeHtml(key)} <span class="text-xs text-gray-500">(${uniqueValues.length})</span></div>
									<div class="text-gray-600 text-sm">
										${uniqueValues.map((v) => `<div class="pl-2 py-1">• ${escapeHtml(v)}</div>`).join("")}
									</div>
								</div>
							`;
							})
							.join("")}
					</div>
				</div>
			`;
		} else {
			detailsHTML = '<span class="text-gray-400">Loading...</span>';
		}

		row.innerHTML = `
			<td class="py-3 px-4 font-medium">${escapeHtml(material.code)}</td>
			<td class="py-3 px-4">
				<div class="flex flex-wrap gap-1">
					${keysList || '<span class="text-gray-400">No keys</span>'}
				</div>
			</td>
			<td class="py-3 px-4">
				${detailsHTML}
			</td>
			<td class="py-3 px-4">${escapeHtml(formatDate(material.created_at))}</td>
			<td class="py-3 px-4">
				<div class="flex gap-2 justify-center">
					<button class="text-blue-600 hover:text-blue-800 edit-btn" data-id="${material.id}">
						<i class="fas fa-edit"></i>
					</button>
					<button class="text-red-600 hover:text-red-800 delete-btn" data-id="${material.id}">
						<i class="fas fa-trash"></i>
					</button>
				</div>
			</td>
		`;

		tbody.appendChild(row);
	});

	// Add event listeners for toggle buttons
	document.querySelectorAll(".toggle-details-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			const id = this.dataset.id;
			const detailCount = this.dataset.count;
			const detailsDiv = document.getElementById(`details-${id}`);
			const icon = this.querySelector("i");

			if (detailsDiv.classList.contains("hidden")) {
				detailsDiv.classList.remove("hidden");
				icon.classList.remove("fa-chevron-down");
				icon.classList.add("fa-chevron-up");
				this.innerHTML = `<i class="fas fa-chevron-up mr-1"></i> Hide Details (${detailCount})`;
			} else {
				detailsDiv.classList.add("hidden");
				icon.classList.remove("fa-chevron-up");
				icon.classList.add("fa-chevron-down");
				this.innerHTML = `<i class="fas fa-chevron-down mr-1"></i> Show Details (${detailCount})`;
			}
		});
	});

	document.querySelectorAll(".edit-btn").forEach((btn) => {
		btn.addEventListener("click", () => {
			const id = btn.dataset.id;
			editMaterial(id);
		});
	});

	document.querySelectorAll(".delete-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			const id = this.dataset.id;
			deleteMaterial(id);
		});
	});
}

function renderPagination() {
	const totalPages = Math.ceil(totalCount / itemsPerPage);
	const paginationContainer = document.getElementById("materials-pagination");

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

function openMaterialModal(materialId = null) {
	const modal = document.getElementById("material-modal");
	const title = document.getElementById("material-modal-title");
	const form = document.getElementById("material-form");

	if (materialId) {
		title.textContent = "Edit Material";
		const material = materials.find((m) => m.id === parseInt(materialId));
		if (material) {
			populateMaterialForm(material);
		}
	} else {
		title.textContent = "Add Material";
		form.reset();
		document.getElementById("material-id").value = "";
	}

	modal.style.display = "flex";
}

function closeMaterialModal() {
	document.getElementById("material-modal").style.display = "none";
}

function populateMaterialForm(material) {
	document.getElementById("material-id").value = material.id;
	document.getElementById("material-code").value = material.code;
}

function handleMaterialFormSubmit(e) {
	e.preventDefault();

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const materialId = document.getElementById("material-id").value;
	const formData = {
		code: document.getElementById("material-code").value,
	};

	if (materialId) {
		formData.id = parseInt(materialId);
		updateMaterial(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	} else {
		createMaterial(formData).finally(() => {
			setButtonLoading(submitBtn, false);
		});
	}
}

async function createMaterial(materialData) {
	try {
		const response = await fetch("/api/materials/create", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(materialData),
		});
		await handleApiResponse(response);

		showNotification("Material created successfully!", "success");
		closeMaterialModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function updateMaterial(materialData) {
	try {
		const response = await fetch("/api/materials/update", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(materialData),
		});
		await handleApiResponse(response);

		showNotification("Material updated successfully!", "success");
		closeMaterialModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function deleteMaterial(materialId) {
	if (!confirm("Are you sure you want to delete this material?")) return;

	const deleteBtn = document.querySelector(`.delete-btn[data-id="${materialId}"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch("/api/materials/delete", {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: parseInt(materialId) }),
		});
		await handleApiResponse(response);

		showNotification("Material deleted successfully!", "success");
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		if (deleteBtn) setButtonLoading(deleteBtn, false);
	}
}

function editMaterial(materialId) {
	openMaterialModal(materialId);
}

async function exportToExcel() {
	const exportBtn = document.getElementById("materials-export-btn");
	setButtonLoading(exportBtn, true);

	try {
		const params = new URLSearchParams();
		const codeFilter = document.getElementById("filter-material-code").value;
		const keyFilter = document.getElementById("filter-material-key").value;
		const hasDescriptions = document.getElementById("filter-has-descriptions").value;

		if (codeFilter) params.append("code", codeFilter);
		if (keyFilter) params.append("key", keyFilter);
		if (hasDescriptions === "yes") params.append("has_descriptions", "true");
		if (hasDescriptions === "no") params.append("has_descriptions", "false");

		const response = await fetch(`/api/materials/filter?${params}`);
		const result = await handleApiResponse(response);
		const filteredMaterials = result.data;

		if (filteredMaterials.length === 0) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filteredMaterials.map((material) => {
			const materialDetails = Object.entries(material.material_details || {})
				.map(([key, value]) => `${key}: ${value}`)
				.join("; ");
			return {
				"Material Code": material.code,
				"Material Keys": material.key,
				"Material Details": materialDetails,
				"Created At": formatDate(material.created_at),
			};
		});

		const worksheet = XLSX.utils.json_to_sheet(data);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, worksheet, "Materials");
		const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
		saveAsExcel(excelBuffer, "materials.xlsx");

		showNotification("Materials exported successfully!", "success");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(exportBtn, false);
	}
}

function setupEventListeners() {
	document.getElementById("materials-apply-filter").addEventListener("click", applyFilters);
	document.getElementById("materials-clear-filter").addEventListener("click", clearFilters);
	document.getElementById("add-material-btn").addEventListener("click", () => openMaterialModal());
	document.getElementById("close-material-modal").addEventListener("click", closeMaterialModal);
	document.getElementById("cancel-material-btn").addEventListener("click", closeMaterialModal);
	document.getElementById("material-form").addEventListener("submit", handleMaterialFormSubmit);

	const exportBtn = document.getElementById("materials-export-btn");
	if (exportBtn) {
		exportBtn.addEventListener("click", exportToExcel);
	}

	document.getElementById("per-page").addEventListener("change", function () {
		itemsPerPage = parseInt(this.value);
		currentPage = 1;
		applyFilters();
	});
}
