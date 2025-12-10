/** @format */

let currentUser = null;
let machines = [];
let processOrders = [];
let flagReasons = [];
let currentJob = null;
let activeJobs = [];

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	await loadCurrentUser();
	await loadMachines();
	await loadFlagReasons();
	setupEventListeners();
	autoSelectShift();
}

async function loadCurrentUser() {
	try {
		const response = await fetch("/api/users/me");
		currentUser = await handleApiResponse(response);
	} catch (error) {
		showNotification(error.message, "error");
	}
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
	const processOrderSelect = document.getElementById("process-order");
	setSelectLoading(processOrderSelect, true);

	const machineSelect = document.getElementById("machine");
	const selectedMachineId = machineSelect.value;

	if (!selectedMachineId) {
		processOrderSelect.innerHTML = '<option value="">Select machine first</option>';
		setSelectLoading(processOrderSelect, false);
		return;
	}

	const selectedMachine = machines.find((m) => m.id == selectedMachineId);
	if (!selectedMachine || !selectedMachine.section_order_types) {
		processOrderSelect.innerHTML = '<option value="">No order type configured for section</option>';
		setSelectLoading(processOrderSelect, false);
		return;
	}

	const today = new Date().toISOString().split("T")[0];
	const orderType = selectedMachine.section_order_types;
	const routingCode = selectedMachine.label;

	try {
		const response = await fetch(`/api/process_order?order_type=${orderType}&routing_code=${routingCode}&posting_date=${today}`);
		const result = await handleApiResponse(response);

		processOrderSelect.innerHTML = '<option value="">Select Process Order</option>';

		if (result.value && Array.isArray(result.value)) {
			result.value.forEach((po) => {
				const option = document.createElement("option");
				option.value = po.ProductionOrder;
				option.textContent = `${po.ProductionOrder}`;
				option.setAttribute(
					"data-po",
					JSON.stringify({
						po: po.ProductionOrder,
						sku: po.Material || "-",
						materials: po.MaterialText || "-",
						planned_films: po.TotalQuantity || "0",
					})
				);
				processOrderSelect.appendChild(option);
			});
		} else {
			processOrderSelect.innerHTML = '<option value="">No process orders found</option>';
		}
	} catch (error) {
		showNotification(error.message, "error");
		processOrderSelect.innerHTML = '<option value="">Failed to load POs</option>';
	} finally {
		setSelectLoading(processOrderSelect, false);
	}
}

async function loadFlagReasons() {
	try {
		const response = await fetch("/api/lookups/flag-reasons");
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
					machineSelect.appendChild(option);
				});
			})
			.catch((error) => {
				console.error("Error loading machines:", error);
				showNotification("Failed to load machines", "error");
			});
	}
}

function populateFlagReasons() {
	const flagReasonSelect = document.getElementById("flag-reason");
	flagReasonSelect.innerHTML = '<option value="">Select Reason</option>';

	flagReasons.forEach((reason) => {
		const option = document.createElement("option");
		option.value = reason.id;
		option.textContent = reason.name;
		flagReasonSelect.appendChild(option);
	});
}

function setupEventListeners() {
	document.getElementById("machine").addEventListener("change", handleMachineChange);
	document.getElementById("process-order").addEventListener("change", handleProcessOrderChange);
	document.getElementById("change-machine-btn").addEventListener("click", resetToMachineSelection);
	document.getElementById("input-form").addEventListener("submit", handleInputSubmit);
	document.getElementById("output-form").addEventListener("submit", handleOutputSubmit);
	document.getElementById("refresh-btn").addEventListener("click", refreshProductionData);
	document.getElementById("active-job").addEventListener("change", handleActiveJobChange);
	document.getElementById("start-new-btn").addEventListener("click", startNewProduction);
}

async function handleMachineChange(e) {
	const machineId = e.target.value;
	const poSection = document.getElementById("po-section");
	const inputSection = document.getElementById("input-section");
	const outputSection = document.getElementById("output-section");
	const activeProduction = document.getElementById("active-production");
	const jobDetails = document.getElementById("job-details");

	if (machineId) {
		if (poSection) poSection.style.display = "block";
		clearProcessOrderDetails();
		await loadProcessOrders();
		await loadActiveJobs(machineId);
	} else {
		if (poSection) poSection.style.display = "none";
		if (inputSection) inputSection.style.display = "none";
		if (outputSection) outputSection.style.display = "none";
		if (activeProduction) activeProduction.style.display = "none";
		if (jobDetails) jobDetails.style.display = "none";
		clearProcessOrderDetails();
	}
}

function clearProcessOrderDetails() {
	const processOrderSelect = document.getElementById("process-order");
	const jobDetails = document.getElementById("job-details");
	const inputForm = document.getElementById("input-form");

	if (processOrderSelect) {
		processOrderSelect.innerHTML = '<option value="">Select Process Order</option>';
		processOrderSelect.disabled = false;
	}
	if (jobDetails) jobDetails.style.display = "none";
	if (inputForm) {
		inputForm.reset();
		inputForm.querySelectorAll("input, select").forEach((el) => {
			el.disabled = false;
		});
		document.querySelector("#input-form button[type='submit']").disabled = true;
		document.getElementById("start-new-btn").style.display = "none";
	}
}

function handleProcessOrderChange(e) {
	const selectedOption = e.target.options[e.target.selectedIndex];
	const jobDetails = document.getElementById("job-details");
	const inputSection = document.getElementById("input-section");
	const startNewBtn = document.getElementById("start-new-btn");

	if (selectedOption.value) {
		const poData = JSON.parse(selectedOption.getAttribute("data-po"));
		displayJobDetails(poData);
		if (jobDetails) jobDetails.style.display = "block";
		if (inputSection) inputSection.style.display = "block";
		if (startNewBtn) startNewBtn.style.display = "block";
	} else {
		if (jobDetails) jobDetails.style.display = "none";
		if (inputSection) inputSection.style.display = "none";
		if (startNewBtn) startNewBtn.style.display = "none";
	}
}

function displayJobDetails(poData) {
	const jobSku = document.getElementById("job-sku");
	const jobFilm = document.getElementById("job-film");
	const jobMaterials = document.getElementById("job-materials");

	if (jobSku) jobSku.textContent = poData.sku || "-";
	if (jobFilm) jobFilm.textContent = poData.materials || "-";
	if (jobMaterials) jobMaterials.textContent = poData.planned_films || "-";
}

async function loadActiveJobs(machineId) {
	try {
		const response = await fetch(`/api/jobs/filter?machine_id=${machineId}&per_page=100`);
		const result = await handleApiResponse(response);
		activeJobs = result.data || result;

		const activeJobSelect = document.getElementById("active-job");
		activeJobSelect.innerHTML = '<option value="">No active job</option>';

		const activeJobsList = activeJobs.filter((job) => !job.completed_at);

		if (activeJobsList.length > 0) {
			activeJobsList.forEach((job) => {
				const option = document.createElement("option");
				option.value = job.id;
				option.textContent = `Job #${job.id} - ${job.batch_roll_no}`;
				option.setAttribute("data-job", JSON.stringify(job));
				activeJobSelect.appendChild(option);
			});
			activeJobSelect.disabled = false;

			// if (activeJobsList.length === 1) {
			currentJob = activeJobsList[0];
			activeJobSelect.value = currentJob.id;
			populateActiveJobDetails(currentJob);
			enableOutputForm();
			loadJobRolls(currentJob.id);
			activeJobSelect.dispatchEvent(new Event("change"));
			// }
		} else {
			currentJob = null;
			activeJobSelect.disabled = true;
			disableOutputForm();
		}
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function handleActiveJobChange(e) {
	const selectedOption = e.target.options[e.target.selectedIndex];

	if (selectedOption.value) {
		const jobData = JSON.parse(selectedOption.getAttribute("data-job"));
		currentJob = jobData;

		const processOrderSelect = document.getElementById("process-order");
		const matchingOption = Array.from(processOrderSelect.options).find((opt) => opt.value === jobData.production_order);
		if (matchingOption) {
			processOrderSelect.value = jobData.production_order;
			processOrderSelect.dispatchEvent(new Event("change"));
		} else {
			populateActiveJobDetails(jobData);
			enableOutputForm();
			loadJobRolls(jobData.id);
		}
	} else {
		currentJob = null;
		disableOutputForm();
	}
}

function populateActiveJobDetails(jobData) {
	const processOrderSelect = document.getElementById("process-order");
	const jobDetails = document.getElementById("job-details");
	const inputForm = document.getElementById("input-form");
	const inputSection = document.getElementById("input-section");
	const startNewBtn = document.getElementById("start-new-btn");

	if (processOrderSelect) {
		processOrderSelect.value = jobData.production_order;
		processOrderSelect.disabled = true;
	}

	if (jobDetails && inputSection) {
		jobDetails.style.display = "block";
		inputSection.style.display = "block";
	}

	if (inputForm) {
		document.getElementById("batch-roll-no").value = jobData.batch_roll_no || "";
		document.getElementById("start-weight").value = jobData.start_weight || "";
		document.getElementById("start-meter").value = jobData.start_meter || "";
		document.getElementById("shift").value = jobData.shift_id || "";

		inputForm.querySelectorAll("input, select").forEach((el) => {
			el.disabled = true;
		});
		document.querySelector("#input-form button[type='submit']").disabled = true;
	}

	if (startNewBtn) startNewBtn.style.display = "block";
}

function startNewProduction() {
	const inputForm = document.getElementById("input-form");
	const processOrderSelect = document.getElementById("process-order");
	const startNewBtn = document.getElementById("start-new-btn");

	if (inputForm) {
		inputForm.reset();
		inputForm.querySelectorAll("input, select").forEach((el) => {
			el.disabled = false;
		});
		document.querySelector("#input-form button[type='submit']").disabled = false;
	}

	if (processOrderSelect) {
		processOrderSelect.disabled = false;
	}

	if (startNewBtn) startNewBtn.style.display = "none";

	currentJob = null;
	disableOutputForm();

	const activeJobSelect = document.getElementById("active-job");
	if (activeJobSelect) activeJobSelect.value = "";
}

function enableOutputForm() {
	const outputRollNo = document.getElementById("output-roll-no");
	const outputSubmitBtn = document.querySelector("#output-form button[type='submit']");
	const outputSection = document.getElementById("output-section");
	const activeProduction = document.getElementById("active-production");

	if (outputRollNo) outputRollNo.value = generateOutputRollNo();
	if (outputSubmitBtn) outputSubmitBtn.disabled = false;
	if (outputSection) outputSection.style.display = "block";
	if (activeProduction) activeProduction.style.display = "block";
}

function disableOutputForm() {
	const outputRollNo = document.getElementById("output-roll-no");
	const outputSubmitBtn = document.querySelector("#output-form button[type='submit']");
	const outputSection = document.getElementById("output-section");
	const activeProduction = document.getElementById("active-production");

	if (outputRollNo) outputRollNo.value = "";
	if (outputSubmitBtn) outputSubmitBtn.disabled = true;
	if (outputSection) outputSection.style.display = "none";
	if (activeProduction) activeProduction.style.display = "none";
}

function generateOutputRollNo() {
	const timestamp = new Date().getTime();
	const random = Math.floor(Math.random() * 1000);
	return `LP3-${timestamp.toString().slice(-3)}${random}`;
}

async function handleInputSubmit(e) {
	e.preventDefault();
	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const formData = {
		shift_id: parseInt(document.getElementById("shift").value),
		production_order: document.getElementById("process-order").value,
		batch_roll_no: document.getElementById("batch-roll-no").value,
		start_weight: parseFloat(document.getElementById("start-weight").value),
		start_meter: parseFloat(document.getElementById("start-meter").value),
		machine_id: parseInt(document.getElementById("machine").value),
	};

	try {
		const newJob = await createJob(formData);
		currentJob = newJob;

		showNotification("Job started successfully!", "success");
		autoSelectShift();

		const activeJobSelect = document.getElementById("active-job");
		const option = document.createElement("option");
		option.value = newJob.id;
		option.textContent = `Job #${newJob.id} - ${newJob.batch_roll_no}`;
		option.setAttribute("data-job", JSON.stringify(newJob));
		activeJobSelect.appendChild(option);
		activeJobSelect.disabled = false;
		activeJobSelect.value = newJob.id;

		populateActiveJobDetails(newJob);
		enableOutputForm();
		loadJobRolls(newJob.id);
	} catch (error) {
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

	const formData = {
		output_roll_no: document.getElementById("output-roll-no").value,
		final_meter: parseFloat(document.getElementById("final-meter").value),
		number_of_flags: parseInt(document.getElementById("flag-count").value) || 0,
		flag_reason_id: document.getElementById("flag-reason").value ? parseInt(document.getElementById("flag-reason").value) : null,
		final_weight: 0,
		job_id: currentJob.id,
	};

	try {
		await createRoll(formData);
		showNotification("Output roll created successfully!", "success");
		e.target.reset();
		document.getElementById("output-roll-no").value = generateOutputRollNo();
		loadJobRolls(currentJob.id);
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(submitBtn, false);
	}
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
	const response = await fetch("/api/rolls/create", {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(rollData),
	});
	return await handleApiResponse(response);
}

async function loadJobRolls(jobId) {
	try {
		const response = await fetch(`/api/rolls/filter?job_id=${jobId}&per_page=10`);
		const result = await handleApiResponse(response);
		const rolls = result.data || result;
		renderRollsTable(rolls);
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function renderRollsTable(rolls) {
	const tbody = document.getElementById("rolls-table-body");

	if (!rolls || rolls.length === 0) {
		tbody.innerHTML = '<tr><td colspan="5" class="text-center text-gray-500 py-4">No rolls found</td></tr>';
		return;
	}

	tbody.innerHTML = rolls
		.map(
			(roll) => `
        <tr class="hover:bg-gray-50">
            <td class="py-3 px-4 font-medium">${escapeHtml(roll.output_roll_no)}</td>
            <td class="py-3 px-4">
                <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
					roll.final_weight === 0
						? "bg-yellow-100 text-yellow-800"
						: roll.number_of_flags > 0
						? "bg-red-100 text-red-800"
						: "bg-green-100 text-green-800"
				}">
                    ${roll.final_weight === 0 ? "Pending" : roll.number_of_flags > 0 ? "Flagged" : "Completed"}
                </span>
            </td>
            <td class="py-3 px-4 text-center">${formatWeight(roll.final_weight || 0)}</td>
            <td class="py-3 px-4 text-center">${formatMeter(roll.final_meter || 0)}</td>
            <!--<td class="py-3 px-4">${escapeHtml(currentJob?.batch_roll_no || "-")}</td>-->
            <!--<td class="py-3 px-4">${escapeHtml(flagReasons.find((r) => r.id === roll.flag_reason_id)?.name || "-")}</td>-->
            <td class="py-3 px-4 text-center">${roll.number_of_flags || 0}</td>
        </tr>
    `
		)
		.join("");
}

function resetToMachineSelection() {
	const machineSelect = document.getElementById("machine");
	const poSection = document.getElementById("po-section");
	const inputSection = document.getElementById("input-section");
	const outputSection = document.getElementById("output-section");
	const activeProduction = document.getElementById("active-production");
	const jobDetails = document.getElementById("job-details");

	if (machineSelect) machineSelect.value = "";
	if (poSection) poSection.style.display = "none";
	if (inputSection) inputSection.style.display = "none";
	if (outputSection) outputSection.style.display = "none";
	if (activeProduction) activeProduction.style.display = "none";
	if (jobDetails) jobDetails.style.display = "none";

	clearProcessOrderDetails();
	currentJob = null;
}

function refreshProductionData() {
	const machineId = document.getElementById("machine").value;
	if (machineId) {
		loadActiveJobs(machineId);
		if (currentJob) {
			loadJobRolls(currentJob.id);
		}
	}
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
