/** @format */
let machines = [];
let processOrders = {};
let flagReasons = [];
let currentJob = null;
const PRINTING_FILM_KEY = ["BOPP", "BOPA", "PET", "FILM", "MATTBOPP"];

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	await loadMachines();
	await loadFlagReasons();
	setupEventListeners();
	setUseRollVisible(false);
	setAddRollVisible(false);
}

async function loadMachines() {
	try {
		const response = await fetch("/api/machines");
		machines = await handleApiResponse(response);
		populateMachineSelect();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function loadProcessOrders() {
	// Prevent race condition: use a request token to ensure only the latest fetch result is processed
	if (!window._poRequestToken) window._poRequestToken = 0;
	const myToken = ++window._poRequestToken;

	const poSelect = document.getElementById("process-order");
	const shiftSelect = document.getElementById("shift-select");
	const dateSelect = document.getElementById("date-select");
	const machineSelect = document.getElementById("machine");
	const selectedMachineId = machineSelect.value;

	poSelect.innerHTML = '<option value="">Loading process order...</option>';
	poSelect.disabled = true;

	if (!selectedMachineId) {
		resetForm();
		poSelect.innerHTML = '<option value="">Select machine first</option>';
		poSelect.disabled = true;
		return;
	}

	if (!shiftSelect.value || !dateSelect.value) {
		resetForm();
		poSelect.innerHTML = '<option value="">Select shift and date first</option>';
		poSelect.disabled = true;
		return;
	}

	const selectedMachine = machines.find((m) => m.id == selectedMachineId);
	if (!selectedMachine || !currentUser || !currentUser.section_ids) {
		resetForm();
		poSelect.innerHTML = '<option value="">No section assigned</option>';
		poSelect.disabled = true;
		return;
	}

	let line = null;
	if (selectedMachine.name) {
		const lineMatch = selectedMachine.name.match(/LINE(\d+)/i);
		if (lineMatch) {
			line = lineMatch[1];
		}
	}

	const sectionIds = currentUser.section_ids.join(",");
	const url = `/api/process_order?section_ids=${sectionIds}${line ? `&line=${line}` : ""}&shift=${shiftSelect.value}&posting_date=${
		dateSelect.value
	}`;

	try {
		const response = await fetch(url);
		// Only process if this is the latest request
		if (myToken !== window._poRequestToken) return;
		const result = await handleApiResponse(response);

		processOrders = {};

		if (result.data && Array.isArray(result.data) && result.data.length > 0) {
			poSelect.innerHTML = "";
			result.data.forEach((po) => {
				processOrders[po.process_order] = {
					process_order: po.process_order,
					description: po.description || "-",
					material_details: po.material_details || {},
					material_numbers: po.material_numbers || {},
				};
				const option = document.createElement("option");
				option.value = po.process_order;
				option.textContent = `${po.process_order} - ${po.description || "-"}`;
				poSelect.appendChild(option);
			});
			poSelect.disabled = false;
			// Do not auto-select if only one process order. Prompt user to select.
			const promptOption = document.createElement("option");
			promptOption.value = "";
			promptOption.textContent = "Select Process Order Now";
			promptOption.disabled = true;
			promptOption.selected = true;
			poSelect.insertBefore(promptOption, poSelect.firstChild);
		} else {
			poSelect.innerHTML = '<option value="">No process order found for this date and shift</option>';
			poSelect.disabled = true;
		}
	} catch (error) {
		if (myToken !== window._poRequestToken) return;
		showNotification(error.message, "error");
		poSelect.innerHTML = '<option value="">Failed to load POs</option>';
		poSelect.disabled = true;
	}
}

function updateInputTitle(poData) {
	const inputTitle = document.getElementById("input-title");
	const materialSelect = document.getElementById("consuming-material");

	if (!inputTitle || !materialSelect) return;

	const materialDetails = poData.material_details || {};
	const materialNumbers = poData.material_numbers || {};

	materialSelect.innerHTML = '<option value="">Select Material</option>';
	inputTitle.textContent = "Currently Consuming (Input)";

	for (const key of PRINTING_FILM_KEY) {
		if (materialDetails[key]) {
			const details = materialDetails[key]
				.split(",")
				.map((item) => item.trim())
				.filter((item) => item !== "");

			details.forEach((detail) => {
				const option = document.createElement("option");
				option.value = detail;
				option.textContent = detail;
				option.dataset.materialNumber = materialNumbers[key];
				materialSelect.appendChild(option);
			});

			break;
		}
	}

	if (materialSelect.children.length <= 1) {
		materialSelect.innerHTML = '<option value="">No material available</option>';
	}
}

async function loadMaterialBatches(materialDetail, materialNumber) {
	const batchSelect = document.getElementById("batch-roll-no");
	const startWeight = document.getElementById("start-weight");
	const startMeter = document.getElementById("start-meter");

	batchSelect.innerHTML = '<option value="">Loading batches...</option>';
	batchSelect.disabled = true;
	startWeight.value = "";

	try {
		const response = await fetch(`/api/materials-availability?material_number=${encodeURIComponent(materialNumber)}&storage_location=DW01`);
		const batches = await handleApiResponse(response);

		batchSelect.innerHTML = '<option value="">Select Batch</option>';

		batches.forEach(([batch, weight]) => {
			const option = document.createElement("option");
			option.value = batch;
			option.textContent = batch;
			option.dataset.weight = weight;
			batchSelect.appendChild(option);
		});

		batchSelect.disabled = false;

		if (batches.length > 0) {
			batchSelect.value = batches[0][0];
			startWeight.value = batches[0][1];
			startMeter.disabled = false;
		}
	} catch (error) {
		batchSelect.innerHTML = '<option value="">Failed to load batches</option>';
		showNotification("Failed to load batches", "error");
	}
}

function updateStartWeight(selectedOption) {
	const startWeight = document.getElementById("start-weight");
	if (selectedOption && selectedOption.dataset.weight) {
		startWeight.value = selectedOption.dataset.weight;
	} else {
		startWeight.value = "";
	}
}

async function loadFlagReasons(sectionId = null) {
	try {
		const url = sectionId ? `/api/lookups/flag-reasons/section/${sectionId}` : "/api/lookups/flag-reasons";
		const response = await fetch(url);
		flagReasons = await handleApiResponse(response);
		populateFlagReasons();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function populateMachineSelect() {
	const machineSelect = document.getElementById("machine");
	machineSelect.innerHTML = '<option value="">Select Machine</option>';

	if (currentUser && currentUser.id) {
		fetch(`/api/machines/filter?user_id=${currentUser.id}&per_page=100`)
			.then((response) => response.json())
			.then((result) => {
				const userMachines = result.data || result;

				userMachines.forEach((machine) => {
					const option = document.createElement("option");
					option.value = machine.id;
					option.textContent = machine.name;
					option.dataset.sectionId = machine.section_id;
					machineSelect.appendChild(option);
				});
			})
			.catch((error) => {
				console.error("Error loading machines:", error);
				showNotification("Failed to load machines", "error");
			});
	}
}

function clearFlagCheckboxes() {
	document.querySelectorAll('#flag-reasons-list input[type="checkbox"]').forEach((cb) => {
		cb.checked = false;
		const messageInput = cb.parentElement.querySelector(".flag-message");
		if (messageInput) {
			messageInput.value = "";
			messageInput.classList.add("hidden");
			messageInput.disabled = true;
		}
	});
}

function populateFlagReasons() {
	const container = document.getElementById("flag-reasons-list");
	if (!container) return;

	container.innerHTML = "";

	flagReasons.forEach((reason) => {
		const wrapper = document.createElement("div");
		wrapper.className = "flex items-center gap-3 p-3 border rounded-lg";

		const checkbox = document.createElement("input");
		checkbox.type = "checkbox";
		checkbox.value = reason.id;
		checkbox.id = `flag-${reason.id}`;
		checkbox.className = "h-4 w-4 text-blue-600 rounded";

		const label = document.createElement("label");
		label.htmlFor = `flag-${reason.id}`;
		label.className = "flex-1 text-gray-700";
		label.textContent = reason.name;

		const countInput = document.createElement("input");
		countInput.type = "number";
		countInput.min = "1";
		countInput.placeholder = "Flag count";
		countInput.className = "form-input w-20 flag-count-input hidden";
		countInput.disabled = true;

		const messageInput = document.createElement("input");
		messageInput.type = "text";
		messageInput.placeholder = "Custom message (optional)";
		messageInput.className = "form-input flex-1 flag-message hidden";
		messageInput.disabled = true;

		checkbox.addEventListener("change", function () {
			countInput.classList.toggle("hidden", !this.checked);
			countInput.disabled = !this.checked;
			if (!this.checked) countInput.value = "";
			messageInput.classList.toggle("hidden", !this.checked);
			messageInput.disabled = !this.checked;
			if (!this.checked) messageInput.value = "";
		});

		wrapper.appendChild(checkbox);
		wrapper.appendChild(label);
		wrapper.appendChild(countInput);
		wrapper.appendChild(messageInput);
		container.appendChild(wrapper);
	});
}

function syncStartNewButton() {
	const submitBtn = document.querySelector("#input-form button[type='submit']");
	const startNewBtn = document.getElementById("start-new-btn");
	if (!submitBtn || !startNewBtn) return;
	const useRollVisible = submitBtn.style.display !== "none";
	if (useRollVisible) startNewBtn.style.display = "none";
}

function setUseRollVisible(visible) {
	const submitBtn = document.querySelector("#input-form button[type='submit']");
	if (!submitBtn) return;
	submitBtn.style.display = visible ? "inline-block" : "none";
	submitBtn.disabled = !visible;
	syncStartNewButton();
}

function setAddRollVisible(visible) {
	const startNewBtn = document.getElementById("start-new-btn");
	if (!startNewBtn) return;
	startNewBtn.style.display = visible ? "inline-block" : "none";
}

function removeClearInputButton() {
	const clearBtn = document.getElementById("clear-input-btn");
	if (clearBtn) clearBtn.remove();
	syncStartNewButton();
}

function addClearInputButton() {
	const submitBtn = document.querySelector("#input-form button[type='submit']");
	if (!submitBtn || document.getElementById("clear-input-btn")) return;

	const clearBtn = document.createElement("button");
	clearBtn.type = "button";
	clearBtn.className = "btn btn-secondary";
	clearBtn.innerHTML = '<i class="fas fa-times"></i> Cancel';
	clearBtn.id = "clear-input-btn";

	clearBtn.addEventListener("click", async function () {
		const originalHTML = clearBtn.innerHTML;
		clearBtn.disabled = true;
		clearBtn.innerHTML = '<i class="fas fa-spinner fa-spin"></i> Deleting...';

		try {
			const response = await fetch("/api/jobs/delete", {
				method: "DELETE",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ id: currentJob.id }),
			});

			if (!response.ok) {
				throw new Error("Failed to delete job");
			}

			showNotification("Job deleted successfully", "success");

			removeClearInputButton();
			enableInputForm();

			const activeJobSelect = document.getElementById("active-job");
			const currentOption = activeJobSelect.querySelector(`option[value=\"${currentJob.id}\"]`);
			if (currentOption) currentOption.remove();

			currentJob = null;
		} catch (error) {
			showNotification(error.message, "error");
			clearBtn.disabled = false;
			clearBtn.innerHTML = originalHTML;
		}
	});

	const buttonContainer = submitBtn.parentElement;
	buttonContainer.insertBefore(clearBtn, submitBtn);
}

function setupEventListeners() {
	document.getElementById("start-new-btn").addEventListener("click", function () {
		if (!currentJob) {
			showNotification("No job selected", "error");
			return;
		}
		document.getElementById("start-new-modal").classList.remove("hidden");
	});

	document.getElementById("machine").addEventListener("change", handleMachineChange);
	document.getElementById("shift-select").addEventListener("change", handleShiftDateChange);
	document.getElementById("date-select").addEventListener("change", handleShiftDateChange);
	document.getElementById("input-form").addEventListener("submit", handleInputSubmit);
	document.getElementById("output-form").addEventListener("submit", handleOutputSubmit);
	document.getElementById("process-order").addEventListener("change", function () {
		const poNumber = this.value;
		const poData = processOrders[poNumber];
		if (poData) {
			updateInputTitle(poData);
			document.getElementById("input-section").style.display = "block";
			setUseRollVisible(true);
			setAddRollVisible(false);
		} else {
			document.getElementById("input-section").style.display = "none";
			document.getElementById("input-form").reset();
			document.getElementById("input-title").textContent = "Currently Consuming (Input)";
			document.getElementById("consuming-material").innerHTML = '<option value="">Select Material</option>';
			document.getElementById("batch-roll-no").innerHTML = '<option value="">Select Material First</option>';
			document.getElementById("batch-roll-no").disabled = true;
			setUseRollVisible(false);
			setAddRollVisible(false);
		}
	});

	document.getElementById("active-job").addEventListener("change", function () {
		if (this.value) {
			const selectedOption = this.options[this.selectedIndex];
			const jobData = JSON.parse(selectedOption.dataset.job);
			loadActiveJobData(jobData);
		}
	});

	document.getElementById("consuming-material").addEventListener("change", function () {
		const batchSelect = document.getElementById("batch-roll-no");
		const startWeight = document.getElementById("start-weight");
		const inputTitle = document.getElementById("input-title");

		batchSelect.innerHTML = '<option value="">Select Material First</option>';
		batchSelect.disabled = true;
		startWeight.value = "";

		if (this.value) {
			inputTitle.textContent = `Currently Consuming (${this.value})`;
			const selectedOption = this.options[this.selectedIndex];
			const materialNumber = selectedOption.dataset.materialNumber;
			loadMaterialBatches(this.value, materialNumber);
		} else {
			inputTitle.textContent = "Currently Consuming (Input)";
		}
	});

	document.getElementById("batch-roll-no").addEventListener("change", function () {
		const selectedOption = this.options[this.selectedIndex];
		updateStartWeight(selectedOption);
	});

	const refreshBtn = document.getElementById("refresh-rolls-btn");
	if (refreshBtn) {
		refreshBtn.addEventListener("click", refreshRecentRolls);
	}

	const today = new Date().toISOString().split("T")[0];
	document.getElementById("date-select").value = today;
}

function resetToMachineSelection() {
	document.getElementById("machine").value = "";
	document.getElementById("machine").disabled = false;
	document.getElementById("shift-select").value = "";
	document.getElementById("shift-select").disabled = false;
	document.getElementById("date-select").disabled = false;
	document.getElementById("process-order").disabled = false;
	document.getElementById("process-order").innerHTML = '<option value="">Select Process Order</option>';

	const materialSelect = document.getElementById("consuming-material");
	const batchSelect = document.getElementById("batch-roll-no");

	materialSelect.disabled = false;
	materialSelect.innerHTML = '<option value="">Select Material</option>';
	batchSelect.disabled = true;
	batchSelect.innerHTML = '<option value="">Select Material First</option>';

	document.getElementById("po-section").style.display = "none";
	document.getElementById("input-section").style.display = "none";
	document.getElementById("output-section").style.display = "none";
	document.getElementById("active-production").style.display = "none";

	document.getElementById("start-weight").value = "";
	document.getElementById("input-title").textContent = "Currently Consuming (Input)";

	currentJob = null;

	document.getElementById("input-form").reset();
	document.getElementById("output-form").reset();

	const submitBtn = document.querySelector("#input-form button[type='submit']");
	if (submitBtn) submitBtn.textContent = "Use Roll";
	setUseRollVisible(false);
	setAddRollVisible(false);

	const clearBtn = document.getElementById("clear-input-btn");
	if (clearBtn) clearBtn.remove();

	const activeJobSelect = document.getElementById("active-job");
	activeJobSelect.innerHTML = '<option value="">No active job</option>';
	activeJobSelect.disabled = true;
}

function cancelStartNew() {
	document.getElementById("start-new-modal").classList.add("hidden");
	document.getElementById("used-weight").value = "";
}

async function confirmStartNew() {
	const endBtn = document.querySelector("#start-new-modal .btn-danger");
	const cancelBtn = document.querySelector("#start-new-modal .btn-secondary");
	const originalEndText = endBtn.textContent;
	const originalCancelText = cancelBtn.textContent;

	endBtn.disabled = true;
	cancelBtn.disabled = true;
	endBtn.innerHTML = '<i class="fas fa-spinner fa-spin"></i> Consuming...';

	try {
		const usedWeightInput = document.getElementById("used-weight");
		let consumedWeight = null;
		let weightUnit = null;

		// Now, collect used weight directly from user
		if (!usedWeightInput.value) {
			showNotification("Please enter the used weight.", "error");
			return;
		}
		consumedWeight = parseFloat(usedWeightInput.value);
		weightUnit = currentJob.start_weight.replace(/[^a-zA-Z]+/g, "").replace(/\s+/g, "");

		const startWeight = parseFloat(currentJob.start_weight);
		if (isNaN(consumedWeight) || isNaN(startWeight)) {
			showNotification("Invalid weight values", "error");
			return;
		}
		if (consumedWeight <= 0) {
			showNotification("Used weight must be greater than 0", "error");
			return;
		}
		if (consumedWeight > startWeight) {
			showNotification("Used weight cannot exceed start weight", "error");
			return;
		}

		if (currentJob && currentJob.id) {
			let materialNumber = currentJob.material_number || currentJob.production_order || "";
			if (!materialNumber && currentJob.selected_material) {
				const materialSelect = document.getElementById("consuming-material");
				const selectedOption = materialSelect.querySelector(`option[value="${currentJob.selected_material}"]`);
				if (selectedOption) {
					materialNumber = selectedOption.dataset.materialNumber || "";
				}
			}

			const postingDate = document.getElementById("date-select").value;
			const postingDateTime = formatDateForAPI(postingDate);

			const endData = {
				id: currentJob.id,
				input_roll_id: currentJob.input_roll_id,
				weight_unit: weightUnit,
				posting_date: postingDateTime,
				material_number: materialNumber,
				consumed_weight: String(consumedWeight),
				batch: currentJob.batch,
				production_order: currentJob.production_order,
			};

			const response = await fetch("/api/input-rolls/end", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(endData),
			});

			await handleApiResponse(response);

			showNotification("Current Roll consumed", "success");

			const activeJobSelect = document.getElementById("active-job");
			const currentOption = activeJobSelect.querySelector(`option[value="${currentJob.id}"]`);
			if (currentOption) {
				currentOption.remove();

				const remainingOptions = activeJobSelect.querySelectorAll("option");
				if (remainingOptions.length <= 1) {
					activeJobSelect.innerHTML = '<option value="">No active job</option>';
					activeJobSelect.disabled = true;
				}
			}
		}

		document.getElementById("start-new-modal").classList.add("hidden");
		usedWeightInput.value = "";

		document.getElementById("date-select").disabled = false;
		document.getElementById("process-order").disabled = false;
		document.getElementById("shift-select").disabled = false;

		enableInputForm();
		setUseRollVisible(true);

		const inputForm = document.getElementById("input-form");
		if (inputForm) {
			inputForm.addEventListener("submit", function lockFieldsOnce(e) {
				document.getElementById("date-select").disabled = true;
				document.getElementById("process-order").disabled = true;
				document.getElementById("shift-select").disabled = true;
				inputForm.removeEventListener("submit", lockFieldsOnce);
			});
		}
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		endBtn.disabled = false;
		cancelBtn.disabled = false;
		endBtn.textContent = originalEndText;
		cancelBtn.textContent = originalCancelText;
	}
}

async function handleShiftDateChange() {
	resetForm();
	await loadProcessOrders();
}

async function handleMachineChange(e) {
	const machineId = e.target.value;

	if (machineId) {
		const selectedOption = e.target.options[e.target.selectedIndex];
		const sectionId = selectedOption.dataset.sectionId;
		if (sectionId) {
			await loadFlagReasons(sectionId);
		}
		const activeJobs = await checkActiveJobs(machineId);

		if (activeJobs.length > 0) {
			populateActiveJobSelect(activeJobs);
			document.getElementById("po-section").style.display = "block";
		} else {
			document.getElementById("po-section").style.display = "block";
			resetForm();
			await loadProcessOrders();

			const activeJobSelect = document.getElementById("active-job");
			activeJobSelect.innerHTML = '<option value="">No active job</option>';
			activeJobSelect.disabled = true;
		}
	} else {
		document.getElementById("po-section").style.display = "none";
		resetForm();
		const activeJobSelect = document.getElementById("active-job");
		activeJobSelect.innerHTML = '<option value="">No active job</option>';
		activeJobSelect.disabled = true;
	}
}

function resetForm() {
	document.getElementById("process-order").innerHTML = '<option value="">Select Process Order</option>';
	document.getElementById("process-order").disabled = true;
	document.getElementById("job-details").style.display = "none";
	document.getElementById("input-section").style.display = "none";
	document.getElementById("input-form").reset();
	const submitBtn = document.querySelector("#input-form button[type='submit']");
	if (submitBtn) submitBtn.textContent = "Use Roll";
	setUseRollVisible(false);
	setAddRollVisible(false);
	document.getElementById("input-title").textContent = "Currently Consuming (Input)";
	document.getElementById("rolls-table-body").innerHTML = `
        <tr>
            <td colspan="5" class="text-center text-gray-500 py-4">No rolls found</td>
        </tr>
    `;
}

function displayJobDetails(poData) {
	const jobMaterials = document.getElementById("job-materials");
	if (!jobMaterials) return;

	jobMaterials.innerHTML = "";
	const materialDetails = poData.material_details || {};

	if (Object.keys(materialDetails).length === 0) {
		const emptyDiv = document.createElement("div");
		emptyDiv.className = "text-gray-500 text-center py-4";
		emptyDiv.textContent = "No material details";
		jobMaterials.appendChild(emptyDiv);
		return;
	}

	const grid = document.createElement("div");
	grid.className = "grid grid-cols-1 md:grid-cols-2 gap-3";

	Object.keys(materialDetails).forEach((key) => {
		const value = materialDetails[key];
		const card = document.createElement("div");
		card.className = "bg-white rounded-lg border border-gray-200 p-3 shadow-sm";

		const header = document.createElement("div");
		header.className = "font-semibold text-gray-800 mb-2 flex items-center gap-2";
		header.innerHTML = `
			<div class="w-8 h-8 rounded-md bg-blue-100 flex items-center justify-center">
				<i class="fas fa-box text-blue-600 text-sm"></i>
			</div>
			<span>${escapeHtml(key)}</span>
		`;

		const content = document.createElement("div");
		content.className = "text-sm text-gray-600";

		if (value.includes(",")) {
			const items = value.split(",").map((item) => item.trim());
			const list = document.createElement("div");
			list.className = "space-y-1";

			items.forEach((item) => {
				const itemDiv = document.createElement("div");
				itemDiv.className = "flex items-start gap-2 py-1";
				itemDiv.innerHTML = `
					<i class="fas fa-check-circle text-green-500 text-xs mt-1"></i>
					<span class="flex-1">${escapeHtml(item)}</span>
				`;
				list.appendChild(itemDiv);
			});
			content.appendChild(list);
		} else {
			const singleItem = document.createElement("div");
			singleItem.className = "flex items-start gap-2 py-1";
			singleItem.innerHTML = `
				<i class="fas fa-check-circle text-green-500 text-xs mt-1"></i>
				<span class="flex-1">${escapeHtml(value)}</span>
			`;
			content.appendChild(singleItem);
		}

		card.appendChild(header);
		card.appendChild(content);
		grid.appendChild(card);
	});

	jobMaterials.appendChild(grid);
}

function enableOutputForm() {
	document.getElementById("final-meter").disabled = false;
	document.getElementById("core-weight").disabled = false;
	document.querySelectorAll('#flag-reasons-list input[type="checkbox"]').forEach((cb) => (cb.disabled = false));
	document.querySelectorAll(".flag-message").forEach((input) => (input.disabled = false));
	document.querySelector("#output-form button[type='submit']").disabled = false;
	document.getElementById("output-section").style.display = "block";
	document.getElementById("active-production").style.display = "block";
}

function disableOutputForm() {
	document.getElementById("final-meter").value = "";
	document.getElementById("final-meter").disabled = true;
	document.getElementById("core-weight").value = "";
	document.getElementById("core-weight").disabled = true;
	document.querySelectorAll('#flag-reasons-list input[type="checkbox"]').forEach((cb) => {
		cb.checked = false;
		cb.disabled = true;
	});
	document.querySelectorAll(".flag-message").forEach((input) => {
		input.value = "";
		input.disabled = true;
		input.classList.add("hidden");
	});
	document.querySelector("#output-form button[type='submit']").disabled = true;
}

function enableInputForm() {
	const materialSelect = document.getElementById("consuming-material");
	const batchSelect = document.getElementById("batch-roll-no");

	materialSelect.disabled = false;
	batchSelect.disabled = true;
	batchSelect.innerHTML = '<option value="">Select Material First</option>';

	document.getElementById("start-meter").disabled = false;
	document.getElementById("start-weight").value = "";
	document.getElementById("start-meter").value = "";

	document.getElementById("input-form").reset();

	const submitBtn = document.querySelector("#input-form button[type='submit']");
	if (submitBtn) submitBtn.textContent = "Use Roll";
	setUseRollVisible(true);

	disableOutputForm();
	syncStartNewButton();
}

function disableInputForm() {
	document.getElementById("consuming-material").disabled = true;
	document.getElementById("batch-roll-no").disabled = true;
	document.getElementById("start-meter").disabled = true;
	document.getElementById("start-weight").disabled = true;
	setUseRollVisible(false);
	setAddRollVisible(true);
	enableOutputForm();
	syncStartNewButton();
}

async function handleInputSubmit(e) {
	e.preventDefault();
	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const batchSelect = document.getElementById("batch-roll-no");
	const selectedBatch = batchSelect.options[batchSelect.selectedIndex];

	const formData = {
		machine_id: parseInt(document.getElementById("machine").value),
		shift_id: parseInt(document.getElementById("shift-select").value),
		production_order: document.getElementById("process-order").value,
		input_roll: {
			job_id: 0, // will be set by backend after job creation
			batch: selectedBatch.value,
			material_number: document.getElementById("consuming-material").selectedOptions[0].dataset.materialNumber,
			start_meter: parseFloat(document.getElementById("start-meter").value),
			start_weight: document.getElementById("start-weight").value,
		},
	};
	if (!formData.production_order) {
		highlightElement("process-order");
		showNotification("Process Order is required", "error");
		setButtonLoading(submitBtn, false);
		return;
	}

	if (!formData.input_roll.material_number) {
		highlightElement("consuming-material");
		showNotification("Material is required", "error");
		setButtonLoading(submitBtn, false);
		return;
	}

	if (!formData.input_roll.batch) {
		highlightElement("batch-roll-no");
		showNotification("Batch Roll No. is required", "error");
		setButtonLoading(submitBtn, false);
		return;
	}

	if (!formData.input_roll.start_weight) {
		highlightElement("start-weight");
		showNotification("Start Weight is required", "error");
		setButtonLoading(submitBtn, false);
		return;
	}

	if (!formData.input_roll.start_meter || formData.input_roll.start_meter <= 0) {
		highlightElement("start-meter");
		showNotification("Start Meter must be greater than 0", "error");
		setButtonLoading(submitBtn, false);
		return;
	}

	if (!formData.machine_id) {
		highlightElement("machine");
		showNotification("Machine is required", "error");
		setButtonLoading(submitBtn, false);
		return;
	}

	try {
		const newJob = await createJob(formData);
		currentJob = newJob;

		showNotification("Job started successfully!", "success");

		addClearInputButton();

		submitBtn.textContent = "Production Started";
		setUseRollVisible(false);
		setAddRollVisible(true);

		document.getElementById("shift-select").disabled = true;
		document.getElementById("date-select").disabled = true;
		document.getElementById("machine").disabled = true;
		document.getElementById("process-order").disabled = true;
		disableInputForm();
	} catch (error) {
		enableInputForm();
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(submitBtn, false);
	}
}

async function handleOutputSubmit(e) {
	e.preventDefault();
	if (!currentJob) {
		showNotification("No active job selected", "error");
		return;
	}

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const selectedFlagWrappers = Array.from(document.querySelectorAll("#flag-reasons-list > div"));
	let selectedFlags = [];
	let totalFlagCount = 0;
	let flagError = false;
	selectedFlagWrappers.forEach((wrapper) => {
		const cb = wrapper.querySelector("input[type='checkbox']");
		const countInput = wrapper.querySelector(".flag-count-input");
		const messageInput = wrapper.querySelector(".flag-message");
		if (cb.checked) {
			const count = parseInt(countInput.value, 10);
			if (!count || count < 1) {
				countInput.classList.add("border-red-500");
				flagError = true;
			} else {
				countInput.classList.remove("border-red-500");
				totalFlagCount += count;
			}
			selectedFlags.push(messageInput && messageInput.value ? `${cb.value}:${messageInput.value}` : cb.value);
		} else {
			countInput.classList.remove("border-red-500");
		}
	});
	if (flagError) {
		showNotification("Please enter a valid flag count (> 0) for each selected flag reason.", "error");
		setButtonLoading(submitBtn, false);
		return;
	}
	const finalMeter = parseFloat(document.getElementById("final-meter").value);
	const coreWeight = parseFloat(document.getElementById("core-weight").value);
	if (!finalMeter || finalMeter <= 0) {
		highlightElement("final-meter");
		showNotification("Final Meter must be greater than 0", "error");
		setButtonLoading(submitBtn, false);
		return;
	}
	if (!coreWeight || coreWeight <= 0) {
		highlightElement("core-weight");
		showNotification("Core Weight must be greater than 0", "error");
		setButtonLoading(submitBtn, false);
		return;
	}
	const formData = {
		final_meter: finalMeter,
		flag_reason: selectedFlags.length > 0 ? selectedFlags.join(" | ") : null,
		flag_count: totalFlagCount,
		final_weight: 0,
		core_weight: coreWeight,
		batch: currentJob.batch,
		job_id: currentJob.id,
		machine_id: currentJob.machine_id,
		shift_id: parseInt(document.getElementById("shift-select").value),
		input_roll_id: currentJob.input_roll_id,
		from_input_batch: document.getElementById("batch-roll-no").value,
	};
	try {
		const createdRoll = await createRoll(formData);
		addRollToTable(createdRoll);
		showNotification("Output roll created successfully!", "success");
		e.target.reset();
		clearFlagCheckboxes();
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(submitBtn, false);
	}
}

function addRollToTable(rollData) {
	const tableBody = document.getElementById("rolls-table-body");

	if (tableBody.children.length === 1 && tableBody.firstElementChild.textContent.includes("No rolls found")) {
		tableBody.innerHTML = "";
	}

	const flagCount = typeof rollData.flag_count === "number" ? rollData.flag_count : 0;

	const row = document.createElement("tr");
	row.className = "hover:bg-gray-50";
	row.innerHTML = `
		<td class="py-2 px-3">${escapeHtml(rollData.from_input_batch)}</td>
		<td class="py-2 px-3">${escapeHtml(rollData.output_batch)}</td>
		<td class="py-2 px-3">${rollData.final_meter || "0.00"}</td>
		<td class="py-2 px-3">${flagCount}</td>
	`;

	tableBody.prepend(row);
}

async function createJob(jobData) {
	const response = await fetch("/api/jobs/create", {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(jobData),
	});
	return await handleApiResponse(response);
}

async function createRoll(rollData) {
	const response = await fetch("/api/output-rolls/create", {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(rollData),
	});
	const result = await handleApiResponse(response);

	const startNewBtn = document.getElementById("start-new-btn");
	if (startNewBtn) {
		setUseRollVisible(false);
		setAddRollVisible(true);
		removeClearInputButton();
	}

	return result;
}

function setSelectLoading(selectElement, loading) {
	if (loading) {
		selectElement.disabled = true;
		const originalHTML = selectElement.innerHTML;
		selectElement.setAttribute("data-original-html", originalHTML);
		selectElement.innerHTML = '<option value="">Loading...</option>';
	} else {
		selectElement.disabled = false;
		const originalHTML = selectElement.getAttribute("data-original-html");
		if (originalHTML && selectElement.children.length <= 1) {
			selectElement.innerHTML = originalHTML;
		}
	}
}

async function checkActiveJobs(machineId) {
	try {
		const response = await fetch(`/api/jobs/filter-with-input-rolls?machine_id=${machineId}&status=active`);
		const result = await handleApiResponse(response);
		return result.data || [];
	} catch (error) {
		return [];
	}
}

async function loadActiveJobData(job) {
	currentJob = job;

	const machineSelect = document.getElementById("machine");
	machineSelect.value = job.machine_id;
	machineSelect.disabled = true;

	const shiftSelect = document.getElementById("shift-select");
	shiftSelect.value = job.shift_id;
	shiftSelect.disabled = true;

	const processOrder = document.getElementById("process-order");
	processOrder.value = job.production_order;

	const machine = machines.find((m) => m.id == job.machine_id);
	let line = null;
	if (machine && machine.name) {
		const lineMatch = machine.name.match(/LINE(\d+)/i);
		if (lineMatch) line = lineMatch[1];
	}

	if (currentUser && currentUser.section_ids) {
		const sectionIds = currentUser.section_ids.join(",");
		const url = `/api/process_order?section_ids=${sectionIds}${line ? `&line=${line}` : ""}&shift=${job.shift_id}`;

		try {
			const response = await fetch(url);
			const result = await handleApiResponse(response);

			const poData = result.data?.find((po) => po.process_order === job.production_order);
			if (poData) {
				processOrders[poData.process_order] = poData;
				const poSelect = document.getElementById("process-order");
				poSelect.innerHTML = "";
				const option = document.createElement("option");
				option.value = poData.process_order;
				option.textContent = `${poData.process_order} - ${poData.description || "-"}`;
				poSelect.appendChild(option);
				poSelect.value = poData.process_order;
				poSelect.disabled = true;
				document.getElementById("date-select").value = poData.posting_date;
				document.getElementById("date-select").disabled = true;

				updateInputTitle(poData);

				const materialSelect = document.getElementById("consuming-material");

				let materialFound = false;

				for (const key of PRINTING_FILM_KEY) {
					if (poData.material_details[key]) {
						const details = poData.material_details[key]
							.split(",")
							.map((item) => item.trim())
							.filter((item) => item !== "");

						if (details.length > 0) {
							materialSelect.innerHTML = '<option value="">Select Material</option>';

							const option = document.createElement("option");
							option.value = details[0];
							option.textContent = details[0];
							const materialNumber = poData.material_numbers[key] || "";
							option.dataset.materialNumber = materialNumber;
							materialSelect.appendChild(option);
							currentJob.material_number = materialNumber;
							materialSelect.value = details[0];
							materialSelect.disabled = true;

							document.getElementById("input-title").textContent = `Currently Consuming (${details[0]})`;
							materialFound = true;
							break;
						}
					}
				}

				if (!materialFound) {
					materialSelect.innerHTML = '<option value="">No material available</option>';
					materialSelect.disabled = true;
				}

				const batchSelect = document.getElementById("batch-roll-no");
				batchSelect.innerHTML = `<option value="${job.batch}">${job.batch}</option>`;
				batchSelect.value = job.batch;
				batchSelect.disabled = true;

				document.getElementById("start-weight").value = job.start_weight;
				document.getElementById("start-meter").value = job.start_meter;

				document.getElementById("input-section").style.display = "block";
				document.getElementById("po-section").style.display = "block";

				disableInputForm();
				enableOutputForm();

				await loadJobRolls(job.production_order);
			}
		} catch (error) {
			console.error("Failed to load process order data:", error);
			showNotification("Failed to load job details", "error");
		}
	}
}

async function loadJobRolls(production_order) {
	try {
		const response = await fetch(`/api/output-rolls/filter?production_order=${production_order}&per_page=50`);
		const result = await handleApiResponse(response);

		const tableBody = document.getElementById("rolls-table-body");
		tableBody.innerHTML = "";

	if (result.data && result.data.length > 0) {
		setUseRollVisible(false);
		setAddRollVisible(true);
		result.data.forEach((roll) => {
			const row = document.createElement("tr");
			row.className = "hover:bg-gray-50";

				const flagCount = typeof roll.flag_count === "number" ? roll.flag_count : 0;

				row.innerHTML = `
					<td class="py-2 px-3">${escapeHtml(roll.from_input_batch)}</td>
					<td class="py-2 px-3">${escapeHtml(roll.output_batch)}</td>
					<td class="py-2 px-3">${roll.final_meter || "0.00"}</td>
					<td class="py-2 px-3">${flagCount}</td>
				`;
				tableBody.appendChild(row);
			});
	} else {
		addClearInputButton();
		setUseRollVisible(false);
		setAddRollVisible(true);

			// startNewBtn.style.display = "none";
			tableBody.innerHTML = `
                <tr>
                    <td colspan="5" class="text-center text-gray-500 py-4">No rolls found</td>
                </tr>
            `;

			const outputSubmitBtn = document.querySelector("#output-form button[type='submit']");
			setButtonLoading(outputSubmitBtn, true);

			setButtonLoading(outputSubmitBtn, false);
		}
	} catch (error) {
		console.error("Failed to load rolls:", error);
	}
}

function getActiveProductionOrder() {
	if (currentJob && currentJob.production_order) return currentJob.production_order;
	const poSelect = document.getElementById("process-order");
	if (poSelect && poSelect.value) return poSelect.value;
	return "";
}

async function refreshRecentRolls() {
	const refreshBtn = document.getElementById("refresh-rolls-btn");
	if (refreshBtn) setButtonLoading(refreshBtn, true);
	try {
		const productionOrder = getActiveProductionOrder();
		if (!productionOrder) {
			showNotification("Select a process order to refresh rolls", "error");
			return;
		}
		await loadJobRolls(productionOrder);
	} finally {
		if (refreshBtn) setButtonLoading(refreshBtn, false);
	}
}

function populateActiveJobSelect(activeJobs) {
	const activeJobSelect = document.getElementById("active-job");
	activeJobSelect.innerHTML = '<option value="">No active job</option>';
	activeJobSelect.disabled = true;

	if (activeJobs.length > 0) {
		activeJobSelect.innerHTML = '<option value="">Select Active Job</option>';

		activeJobs.forEach((job) => {
			const option = document.createElement("option");
			option.value = job.id;
			option.textContent = `${job.production_order} - ${job.batch}`;
			option.dataset.job = JSON.stringify(job);
			activeJobSelect.appendChild(option);
		});

		activeJobSelect.disabled = false;

		if (activeJobs.length === 1) {
			activeJobSelect.value = activeJobs[0].id;
			loadActiveJobData(activeJobs[0]);
		}
	}
}

function modifyJobFilterFunction() {
	const jobFilterUrl = "/api/jobs/filter-with-input-rolls?status=active";

	fetch(jobFilterUrl)
		.then((response) => response.json())
		.then((result) => {
			const activeJobs = result.data || [];
			if (activeJobs.length > 0) {
				document.getElementById("active-job").innerHTML = "";

				activeJobs.forEach((job) => {
					const option = document.createElement("option");
					option.value = job.id;
					option.textContent = `${job.production_order} - ${job.batch}`;
					document.getElementById("active-job").appendChild(option);
				});

				document.getElementById("active-job").disabled = false;
				document.getElementById("active-job").value = activeJobs[0].id;
			} else {
				document.getElementById("active-job").innerHTML = '<option value="">No active job</option>';
				document.getElementById("active-job").disabled = true;
			}
		})
		.catch((error) => {
			console.error("Failed to load active jobs:", error);
		});
}
