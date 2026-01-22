/** @format */

// --- VECTOR LABEL, RASTER QR PDF EXPORT ---
async function printRoll(rollId) {
	openPrintModal();

	const printPreview = document.getElementById("print-preview");
	const printLoading = document.getElementById("print-loading");

	if (printPreview) printPreview.classList.add("hidden");
	if (printLoading) printLoading.classList.remove("hidden");

	try {
		const response = await fetch(`/api/rolls/details?id=${rollId}`);
		const result = await handleApiResponse(response);

		document.getElementById("print-preview").innerHTML = `
			<div style="background: #fff; margin: 0; display: flex; justify-content: center; padding: 10px;" id="print-preview-content">
				<div style="width: 400px; height: 580px; border: 2px solid #000; padding: 25px; font-family: Arial, sans-serif; display: flex; flex-direction: column;">
					<div style="text-align: center; margin-bottom: 30px;">
						<div style="font-weight: 900; font-size: 18px; letter-spacing: 0.5px;">INSIGNIA PRODUCTION LABEL</div>
						<div style="font-size: 13px; font-weight: bold; margin-top: 4px;">${escapeHtml(result.process_order_description)}</div>
					</div>
					<!-- QR CODE IN CENTER -->
					<div style="text-align: center; margin: 0 auto 30px auto;">
						<div id="qrcode-container" style="display:block;width:180px;height:180px;margin:0;padding:0;"></div>
						<label style="margin-top: 5px; font-size: 8px; font-weight: bold; display: block; text-transform: uppercase; color: #444;">ROLL NO SCAN</label>
					</div>
					<!-- CONTENT BELOW QR CODE -->
					<div style="display: flex; justify-content: space-between; margin-bottom: 20px;">
						<div style="flex: 1;">
							<label style="font-size: 10px; font-weight: bold; display: block; text-transform: uppercase; color: #444; margin-bottom: 4px;">PRODUCTION ORDER</label>
							<div style="font-size: 12px; font-weight: bold; margin-bottom: 15px;">${escapeHtml(result.production_order)}</div>
							<label style="font-size: 10px; font-weight: bold; display: block; text-transform: uppercase; color: #444; margin-bottom: 4px;">SECTION</label>
							<div style="font-size: 12px; font-weight: bold; margin-bottom: 15px;">${escapeHtml(result.section)}</div>
						</div>
						<div style="flex: 1; text-align: right;">
							<label style="font-size: 10px; font-weight: bold; display: block; text-transform: uppercase; color: #444; margin-bottom: 4px;">MATERIAL NUMBER</label>
							<div style="font-size: 12px; font-weight: bold; margin-bottom: 15px;">${escapeHtml(result.material_number)}</div>
							<label style="font-size: 10px; font-weight: bold; display: block; text-transform: uppercase; color: #444; margin-bottom: 4px;">ROLL NO</label>
							<div style="font-size: 12px; font-weight: bold; margin-bottom: 15px;">${escapeHtml(result.output_roll_no)}</div>
						</div>
					</div>
					<div style="display: flex; margin-bottom: 25px;">
						<div style="flex: 1;">
							<label style="font-size: 10px; font-weight: bold; display: block; text-transform: uppercase; color: #444; margin-bottom: 4px;">FINAL METER</label>
							<div style="font-size: 14px; font-weight: bold;">${formatNumber(result.final_meter)} m</div>
						</div>
						<div style="flex: 1;">
							<label style="font-size: 10px; font-weight: bold; display: block; text-transform: uppercase; color: #444; margin-bottom: 4px;">FINAL WEIGHT</label>
							<div style="font-size: 14px; font-weight: bold;">${formatNumber(result.final_weight)} kg</div>
						</div>
					</div>
					<div style="margin-top: auto; display: flex; justify-content: space-between; padding-bottom: 5px;">
						<div>
							<label style="font-size: 10px; font-weight: bold; display: block; text-transform: uppercase; color: #444; margin-bottom: 4px;">TIMESTAMP</label>
							<div style="font-size: 11px; font-family: monospace; font-weight: bold;">${escapeHtml(result.created_at)}</div>
						</div>
						<div style="text-align: right;">
							<label style="font-size: 10px; font-weight: bold; display: block; text-transform: uppercase; color: #444; margin-bottom: 4px;">REMARK</label>
							<div style="font-size: 11px; font-family: monospace; font-weight: bold;">
								${result.flag_count > 0 ? `${result.flag_count} Flags` : "GOOD"}
							</div>
						</div>
					</div>
				</div>
			</div>
			<div class="mt-6 flex justify-center gap-3">
				<button id="print-pdf-btn" class="btn btn-primary"><i class="fas fa-print mr-2"></i> Print Label</button>
				<button id="close-print-btn" class="btn btn-secondary"><i class="fas fa-times mr-2"></i> Close</button>
			</div>
		`;

		if (printLoading) printLoading.classList.add("hidden");
		if (printPreview) printPreview.classList.remove("hidden");
		setTimeout(() => {
			const qrContainer = document.getElementById("qrcode-container");
			if (qrContainer) {
				const flagsCount = result.flag_reason
					? result.flag_reason
							.split("|")
							.map((s) => s.trim())
							.filter((s) => s.length > 0).length
					: 0;
				const qrPayload = {
					"po no": "PO-" + result.production_order,
					"material no": "MA-" + result.material_number,
					material: result.process_order_description,
					"roll no": result.output_roll_no,
					flags: flagsCount,
					weight: `${result.final_weight}kg`,
					meter: `${result.final_meter}m`,
					date: result.created_at,
				};
				const qrData = JSON.stringify(qrPayload);
				qrContainer.innerHTML = "";
				const qr = qrcode(0, "L");
				qr.addData(qrData);
				qr.make();
				const moduleCount = qr.getModuleCount();
				const containerSize = Math.min(qrContainer.clientWidth || 180, qrContainer.clientHeight || 180);
				let cellSize = Math.floor(containerSize / moduleCount);
				if (cellSize < 2) cellSize = 2;
				qrContainer.innerHTML = qr.createSvgTag(cellSize, 0);
				const svgEl = qrContainer.querySelector("svg");
				if (svgEl) {
					const size = moduleCount * cellSize;
					svgEl.setAttribute("viewBox", `0 0 ${size} ${size}`);
					svgEl.setAttribute("preserveAspectRatio", "none");
					svgEl.style.width = "100%";
					svgEl.style.height = "100%";
					svgEl.style.display = "block";
					svgEl.style.margin = "0";
					svgEl.style.padding = "0";
				}
			}
		}, 100);

		const { PDFDocument, StandardFonts, rgb } = PDFLib;

		const flagsCount = result.flag_reason
			? result.flag_reason
					.split("|")
					.map((s) => s.trim())
					.filter(Boolean).length
			: 0;

		const qrPayload = {
			"po no": "PO-" + result.production_order,
			"material no": "MA-" + result.material_number,
			material: result.process_order_description,
			"roll no": result.output_roll_no,
			flags: flagsCount,
			weight: `${result.final_weight}kg`,
			meter: `${result.final_meter}m`,
			date: result.created_at,
		};

		const qr = qrcode(0, "L");
		qr.addData(JSON.stringify(qrPayload));
		qr.make();
		const n = qr.getModuleCount();
		const scale = 6;
		const size = n * scale;

		const canvas = document.createElement("canvas");
		canvas.width = size;
		canvas.height = size;
		const ctx = canvas.getContext("2d");
		ctx.fillStyle = "#fff";
		ctx.fillRect(0, 0, size, size);
		ctx.fillStyle = "#000";
		for (let r = 0; r < n; r++) for (let c = 0; c < n; c++) if (qr.isDark(r, c)) ctx.fillRect(c * scale, r * scale, scale, scale);
		const qrPng = await fetch(canvas.toDataURL("image/png")).then((r) => r.arrayBuffer());

		const pdf = await PDFDocument.create();
		const page = pdf.addPage([288, 432]); // 4x6 in @72dpi
		const font = await pdf.embedFont(StandardFonts.Helvetica);
		const fontB = await pdf.embedFont(StandardFonts.HelveticaBold);

		// No border, no main padding

		// Centered title and subtitle
		const centerX = (text, size, fontObj) => (288 - fontObj.widthOfTextAtSize(text, size)) / 2;

		let y = 410;
		const title = "INSIGNIA PRODUCTION LABEL";
		page.drawText(title, { x: centerX(title, 14, fontB), y, size: 14, font: fontB });
		y -= 18;
		const wrap = (t, max = 38) => {
			const w = (t || "").replace(/\s+/g, " ").trim();
			const out = [];
			let line = "";
			for (const word of w.split(" ")) {
				const next = line ? line + " " + word : word;
				if (next.length > max) {
					if (line) out.push(line);
					line = word;
				} else line = next;
			}
			if (line) out.push(line);
			return out;
		};
		// Always show at least one subtitle line
		const subtitleLines = wrap(result.process_order_description, 38);
		if (subtitleLines.length === 0) subtitleLines.push("");
		subtitleLines.slice(0, 2).forEach((ln) => {
			page.drawText(ln, { x: centerX(ln, 11, fontB), y, size: 11, font: fontB });
			y -= 13;
		});

		// Move QR code lower so it does NOT cover title/subtitle
		const img = await pdf.embedPng(qrPng);
		// Move QR code up a bit, and push text blocks below QR code further down
		const qrY = y - 5; // QR code closer to subtitle
		page.drawImage(img, { x: (288 - 150) / 2, y: qrY - 150, width: 150, height: 150 });
		page.drawText("ROLL NO SCAN", { x: 110, y: qrY - 158, size: 7, font: fontB, color: rgb(0.25, 0.25, 0.25) });

		// Adjust y for key-value text blocks to come down more
		let kvStartY = qrY - 200; // move all key-value texts further down

		const leftX = 24,
			rightX = 288 - 24 - 80; // 24px margin from right, 80px block width
		// Increase font sizes for key-value texts
		const kv = (label, value, x, y, align = "left") => {
			const labelFontSize = 10;
			const valueFontSize = 13;
			if (align === "right") {
				const blockWidth = 80;
				const margin = 24;
				const labelWidth = fontB.widthOfTextAtSize(label, labelFontSize);
				const valueWidth = fontB.widthOfTextAtSize(String(value ?? ""), valueFontSize);
				const rightEdge = 288 - margin;
				page.drawText(label, {
					x: rightEdge - labelWidth,
					y,
					size: labelFontSize,
					font: fontB,
					color: rgb(0.25, 0.25, 0.25),
					align: "right",
				});
				page.drawText(String(value ?? ""), {
					x: rightEdge - valueWidth,
					y: y - 15,
					size: valueFontSize,
					font: fontB,
					align: "right",
				});
			} else {
				page.drawText(label, { x, y, size: labelFontSize, font: fontB, color: rgb(0.25, 0.25, 0.25) });
				page.drawText(String(value ?? ""), { x, y: y - 15, size: valueFontSize, font: fontB });
			}
		};

		kv("PRODUCTION ORDER", result.production_order, leftX, kvStartY);
		kv("SECTION", result.section, leftX, kvStartY - 34);
		kv("MATERIAL NUMBER", result.material_number, rightX, kvStartY, "right");
		kv("ROLL NO", result.output_roll_no, rightX, kvStartY - 34, "right");

		kv("FINAL METER", `${formatNumber(result.final_meter)} m`, leftX, kvStartY - 72);
		kv("FINAL WEIGHT", `${formatNumber(result.final_weight)} kg`, rightX, kvStartY - 72, "right");

		// Only show the values for TIMESTAMP and REMARK, no key/label
		const valueFontSize = 13;
		page.drawText(String(result.created_at ?? ""), { x: leftX, y: kvStartY - 152, size: valueFontSize, font });
		page.drawText(result.flag_count > 0 ? `${result.flag_count} Flags` : "GOOD", { x: rightX, y: kvStartY - 152, size: valueFontSize, font });

		const bytes = await pdf.save();
		const a = document.createElement("a");
		const pdfBlob = new Blob([bytes], { type: "application/pdf" });
		a.href = URL.createObjectURL(pdfBlob);
		a.download = `label-${result.output_roll_no || rollId}.pdf`;
		document.getElementById("close-print-btn").addEventListener("click", closePrintModal);
		document.getElementById("print-pdf-btn").addEventListener("click", async function () {
			a.click();
			const reader = new FileReader();
			reader.onloadend = async function () {
				const base64data = reader.result.split(",")[1];
				await fetch("http://localhost:8080/api/app/print", {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({ pdf_data: base64data }),
				});
				showNotification("Label sent to printer", "success");
			};
			reader.readAsDataURL(pdfBlob);
		});
	} catch (error) {
		showNotification(error.message, "error");
		closePrintModal();
	}
}

let rolls = [];
let flagReasons = [];
let users = [];
let showDetails = false;
let currentPage = 1;
let itemsPerPage = 10;
let totalCount = 0;

document.addEventListener("DOMContentLoaded", function () {
	initializePage();
});

async function initializePage() {
	showDetails = localStorage.getItem("rollsShowDetails") === "true";
	const toggle = document.getElementById("details-toggle");
	if (toggle) {
		toggle.checked = showDetails;
		toggle.addEventListener("change", toggleDetails);
	}
	await loadRolls();
	setupEventListeners();
	setTimeout(() => toggleDetails(), 100);
}

function toggleDetails() {
	showDetails = !showDetails;
	const toggle = document.getElementById("details-toggle");
	if (toggle) toggle.checked = showDetails;
	const detailColumns = document.querySelectorAll(".details-column");
	const detailCells = document.querySelectorAll(".detail-cell");
	const allCells = document.querySelectorAll("#rolls-table tbody td");

	if (showDetails) {
		detailColumns.forEach((col) => (col.style.display = "table-cell"));
		detailCells.forEach((cell) => (cell.style.display = "table-cell"));
		allCells.forEach((cell) => {
			if (cell.cellIndex === 3 || cell.cellIndex === 4 || cell.cellIndex === 5) {
				cell.classList.add("text-center");
			} else {
				cell.classList.remove("text-center");
			}
		});
	} else {
		detailColumns.forEach((col) => (col.style.display = "none"));
		detailCells.forEach((cell) => (cell.style.display = "none"));
		allCells.forEach((cell) => cell.classList.remove("text-center"));
	}

	localStorage.setItem("rollsShowDetails", showDetails);
}

async function loadFilterOptions() {
	try {
		const [reasonsResponse, usersResponse] = await Promise.all([
			fetch("/api/lookups/flag-reasons").then(handleApiResponse),
			fetch("/api/users").then(handleApiResponse),
		]);

		flagReasons = reasonsResponse;
		users = usersResponse;

		populateSelect("filter-flag-reason", flagReasons, "name", "All Reasons");
		populateSelect("filter-user", users, "full_name", "All Users");
	} catch (error) {
		showNotification(error.message, "error");
	}
}

async function loadRolls() {
	showLoading(true, "rolls-table");
	try {
		const params = new URLSearchParams();
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/rolls/filter?${params}`);
		const result = await handleApiResponse(response);

		rolls = result.data;
		totalCount = result.total_count;
		updateRollStats(rolls);
		renderRolls(rolls);
		renderPagination();
		updatePerPageOptions(result.total_count);
	} catch (error) {
		document.getElementById("rolls-table-body").innerHTML =
			'<tr><td colspan="8" class="text-center text-red-500 py-4">Failed to load roll records</td></tr>';
		showNotification(error.message, "error");
	} finally {
		showLoading(false, "rolls-table");
	}
}

function updateRollStats(rolls) {
	const totalRecords = rolls.length;
	const completedRolls = rolls.filter((roll) => roll.final_weight > 0).length;
	const pendingRolls = rolls.filter((roll) => roll.final_weight === 0).length;
	const flaggedRolls = rolls.filter((roll) => roll.flag_count > 0).length;

	document.getElementById("total-rolls").textContent = totalRecords;
	document.getElementById("completed-rolls").textContent = completedRolls;
	document.getElementById("pending-rolls").textContent = pendingRolls;
	document.getElementById("flagged-rolls").textContent = flaggedRolls;
}

function renderRolls(rollsToRender) {
	const tbody = document.getElementById("rolls-table-body");
	if (!rollsToRender || rollsToRender.length === 0) {
		tbody.innerHTML = '<tr><td colspan="8" class="text-center text-gray-500 py-4">No roll records found</td></tr>';
		return;
	}

	tbody.innerHTML = "";

	rollsToRender.forEach((roll) => {
		const reason = flagReasons.find((r) => r.id === roll.flag_reason_id);
		const createdBy = users.find((u) => u.id === roll.created_by);
		const isEditable = roll.final_weight === 0;
		const isDeletable = roll.final_weight === 0;

		const row = document.createElement("tr");
		row.className = "hover:bg-gray-50";

		row.innerHTML = `
            <td class="py-3 px-4 detail-cell" ${!showDetails ? 'style="display: none"' : ""}>${escapeHtml(roll.from_batch || "N/A")}</td>
            <td class="py-3 px-4 font-medium detail-cell" ${!showDetails ? 'style="display: none"' : ""}>${escapeHtml(roll.output_roll_no)}</td>
            <td class="py-3 px-4 ${showDetails ? "" : "text-left"}">
				<span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
					roll.final_weight === 0
						? "bg-yellow-100 text-yellow-800"
						: roll.flag_count > 0
							? "bg-red-100 text-red-800"
							: "bg-green-100 text-green-800"
				}">
					${roll.final_weight === 0 ? "Pending" : roll.flag_count > 0 ? "Flagged" : "Completed"}
				</span>
            </td>
            <td class="py-3 px-4 ${showDetails ? "text-center" : ""}">${formatWeight(roll.final_weight || 0)}</td>
            <td class="py-3 px-4 ${showDetails ? "text-center" : ""}">${roll.final_meter || 0} m</td>
			<td class="py-3 px-4 detail-cell ${showDetails ? "text-center" : ""}" ${!showDetails ? 'style="display: none"' : ""}>${
				typeof roll.flag_count === "number" ? roll.flag_count : 0
			}</td>
            <td class="py-3 px-4 ${showDetails ? "" : "text-left"}">${formatDateTime(roll.created_at)}</td>
            <td class="py-3 px-4">
                <div class="flex gap-2">
                    ${
						isEditable
							? `<button class="text-blue-600 hover:text-blue-800 edit-btn" data-id="${roll.id}">
                            <i class="fas fa-edit"></i>
                        </button>`
							: `<button class="text-green-600 hover:text-green-800 print-btn" data-id="${roll.id}">
                            <i class="fas fa-print"></i>
                        </button>`
					}
                    ${
						isDeletable
							? `<button class="text-red-600 hover:text-red-800 delete-btn" data-id="${roll.id}">
                            <i class="fas fa-trash"></i>
                        </button>`
							: ""
					}
                </div>
            </td>
        `;

		if (roll.flag_reason_id) {
			const reason = flagReasons.find((r) => r.id === roll.flag_reason_id);
			if (reason) {
				row.setAttribute("title", `Flag Reason: ${reason.name}`);
				row.classList.add("cursor-help");
			}
		}

		tbody.appendChild(row);
	});

	document.querySelectorAll(".edit-btn").forEach((btn) => {
		btn.addEventListener("click", () => editRoll(btn.dataset.id));
	});

	document.querySelectorAll(".delete-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			deleteRoll(btn.dataset.id);
		});
	});

	document.querySelectorAll(".print-btn").forEach((btn) => {
		btn.addEventListener("click", function () {
			printRoll(btn.dataset.id);
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
	showLoading(true, "rolls-table");

	try {
		const params = new URLSearchParams();
		const searchTerm = document.getElementById("filter-search").value;
		const statusFilter = document.getElementById("filter-status").value;
		// const reasonFilter = document.getElementById("filter-flag-reason").value;
		// const userFilter = document.getElementById("filter-user").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const startDate = document.getElementById("filter-start-date").value;
		const endDate = document.getElementById("filter-end-date").value;

		if (searchTerm) {
			if (!isNaN(searchTerm) && searchTerm.trim() !== "") {
				params.append("job_id", searchTerm);
			} else {
				params.append("output_roll_no", searchTerm);
			}
		}

		if (statusFilter) params.append("status", statusFilter);

		// if (reasonFilter) params.append("flag_reason_id", reasonFilter);
		// if (userFilter) params.append("created_by", userFilter);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (startDate) params.append("start_date", startDate);
		if (endDate) params.append("end_date", endDate);
		params.append("page", currentPage);
		params.append("per_page", itemsPerPage);

		const response = await fetch(`/api/rolls/filter?${params}`);
		const result = await handleApiResponse(response);

		rolls = result.data;
		totalCount = result.total_count;
		updateRollStats(rolls);
		renderRolls(rolls);
		renderPagination();
		updatePerPageOptions(result.total_count);
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(applyBtn, false);
		showLoading(false, "rolls-table");
	}
}

function clearFilters() {
	document.getElementById("filter-search").value = "";
	document.getElementById("filter-status").value = "";
	// document.getElementById("filter-flag-reason").value = "";
	// document.getElementById("filter-user").value = "";
	document.getElementById("filter-shift").value = "";
	document.getElementById("filter-start-date").value = "";
	document.getElementById("filter-end-date").value = "";

	currentPage = 1;
	applyFilters();
}

async function openModal(rollId = null) {
	const modal = document.getElementById("roll-modal");
	const title = document.getElementById("modal-title");
	const form = document.getElementById("roll-form");

	if (rollId) {
		title.textContent = "Update Roll";
		const roll = rolls.find((r) => r.id === parseInt(rollId));
		if (roll) {
			populateForm(roll);
		}
	} else {
		title.textContent = "Add Roll";
		form.reset();
		document.getElementById("roll-id").value = "";
	}

	// Fetch scale weight and set input value (rounded to 3 decimals), disable input
	try {
		const resp = await fetch("http://localhost:8080/api/app/weight");
		if (resp.ok) {
			const data = await resp.json();
			if (data && typeof data.weight !== "undefined") {
				const weight = Math.round((parseFloat(data.weight) + Number.EPSILON) * 1000) / 1000;
				const weightInput = document.getElementById("final-weight");
				if (weightInput) {
					weightInput.value = weight;
					weightInput.disabled = true;
				}
			}
		}
	} catch (e) {
		// If fetch fails, leave input enabled for manual entry
		const weightInput = document.getElementById("final-weight");
		if (weightInput) {
			weightInput.disabled = false;
		}
		showNotification("Could not fetch scale weight. Enter manually.", "warning");
	}

	modal.style.display = "flex";
}

function closeModal() {
	document.getElementById("roll-modal").style.display = "none";
}

function populateForm(roll) {
	document.getElementById("roll-id").value = roll.id;
	document.getElementById("final-weight").value = roll.final_weight || "";
}

function handleFormSubmit(e) {
	e.preventDefault();

	const rollId = document.getElementById("roll-id").value;
	const roll = rolls.find((r) => r.id === parseInt(rollId));

	if (roll && roll.final_weight > 0) {
		showNotification("Only pending rolls can be updated", "warning");
		return;
	}

	const submitBtn = e.target.querySelector('button[type="submit"]');
	setButtonLoading(submitBtn, true);

	const formData = {
		id: parseInt(rollId),
		final_weight: parseFloat(document.getElementById("final-weight").value),
	};

	updateRoll(formData).finally(() => {
		setButtonLoading(submitBtn, false);
	});
}

async function updateRoll(rollData) {
	try {
		const response = await fetch("/api/rolls/update", {
			method: "PUT",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(rollData),
		});
		await handleApiResponse(response);

		showNotification("Roll updated successfully!", "success");
		closeModal();
		await applyFilters();
	} catch (error) {
		showNotification(error.message, "error");
	}
}

function editRoll(rollId) {
	const roll = rolls.find((r) => r.id === parseInt(rollId));

	if (roll.final_weight > 0 && roll.flag_count === 0) {
		showNotification("Completed rolls cannot be edited", "warning");
		return;
	}

	if (roll.final_weight > 0 && roll.flag_count > 0) {
		showNotification("Flagged rolls cannot be edited", "warning");
		return;
	}

	openModal(rollId);
}

async function deleteRoll(rollId) {
	if (!confirm("Are you sure you want to delete this roll?")) return;

	const deleteBtn = document.querySelector(`.delete-btn[data-id="${rollId}"]`);
	if (deleteBtn) setButtonLoading(deleteBtn, true);

	try {
		const response = await fetch("/api/rolls/delete", {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ id: parseInt(rollId) }),
		});
		await handleApiResponse(response);

		showNotification("Roll deleted successfully!", "success");
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
		const searchTerm = document.getElementById("filter-search").value;
		const statusFilter = document.getElementById("filter-status").value;
		// const reasonFilter = document.getElementById("filter-flag-reason").value;
		// const userFilter = document.getElementById("filter-user").value;
		const shiftFilter = document.getElementById("filter-shift").value;
		const startDate = document.getElementById("filter-start-date").value;
		const endDate = document.getElementById("filter-end-date").value;

		if (searchTerm) {
			if (!isNaN(searchTerm) && searchTerm.trim() !== "") {
				params.append("job_id", searchTerm);
			} else {
				params.append("output_roll_no", searchTerm);
			}
		}

		if (statusFilter) {
			if (statusFilter === "pending") {
				params.append("final_weight", "0");
			} else if (statusFilter === "flagged") {
				params.append("number_of_flags_gt", "0");
			} else if (statusFilter === "completed") {
				params.append("final_weight_gt", "0");
				params.append("number_of_flags", "0");
			}
		}

		// if (reasonFilter) params.append("flag_reason_id", reasonFilter);
		// if (userFilter) params.append("created_by", userFilter);
		if (shiftFilter) params.append("shift_id", shiftFilter);
		if (startDate) params.append("start_date", startDate);
		if (endDate) params.append("end_date", endDate);

		const response = await fetch(`/api/rolls/filter?${params}`);
		const result = await handleApiResponse(response);
		const filteredRolls = result.data;

		if (filteredRolls.length === 0) {
			showNotification("No data to export", "warning");
			return;
		}

		const data = filteredRolls.map((roll) => {
			const reason = flagReasons.find((r) => r.id === roll.flag_reason_id);
			const createdBy = users.find((u) => u.id === roll.created_by);

			return {
				"Output Roll No": roll.output_roll_no,
				Status: roll.final_weight === 0 ? "Pending" : roll.flag_count > 0 ? "Flagged" : "Completed",
				Weight: formatWeight(roll.final_weight || 0),
				Meters: roll.final_meter || 0 + " m",
				"Job ID": roll.job_id,
				"Flag Reason": reason?.name || "-",
				"Flag Count": typeof roll.flag_count === "number" ? roll.flag_count : 0,
				"Created By": createdBy?.full_name || "System",
				"Created At": formatDateTime(roll.created_at),
			};
		});

		const worksheet = XLSX.utils.json_to_sheet(data);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, worksheet, "Roll Records");
		const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
		saveAsExcel(excelBuffer, "roll_records.xlsx");

		showNotification("Roll records exported successfully!", "success");
	} catch (error) {
		showNotification(error.message, "error");
	} finally {
		setButtonLoading(exportBtn, false);
	}
}

function openPrintModal() {
	const printModal = document.getElementById("print-modal");
	if (printModal) {
		printModal.style.display = "flex";
		printModal.classList.add("active");
	}
}

function closePrintModal() {
	const printModal = document.getElementById("print-modal");
	if (printModal) {
		printModal.style.display = "none";
		printModal.classList.remove("active");
	}
	const printPreview = document.getElementById("print-preview");
	const printLoading = document.getElementById("print-loading");
	if (printPreview) printPreview.classList.add("hidden");
	if (printLoading) printLoading.classList.add("hidden");
}

function formatNumber(num) {
	return num.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

function setupEventListeners() {
	document.getElementById("apply-filter").addEventListener("click", applyFilters);
	document.getElementById("clear-filter").addEventListener("click", clearFilters);
	document.getElementById("close-modal").addEventListener("click", closeModal);
	document.getElementById("cancel-btn").addEventListener("click", closeModal);
	document.getElementById("roll-form").addEventListener("submit", handleFormSubmit);

	const closePrintModalBtn = document.getElementById("close-print-modal");
	const closePrintBtn = document.getElementById("close-print-btn");
	const printPdfBtn = document.getElementById("print-pdf-btn");

	if (closePrintModalBtn) closePrintModalBtn.addEventListener("click", closePrintModal);
	if (closePrintBtn) closePrintBtn.addEventListener("click", closePrintModal);
	if (printPdfBtn)
		printPdfBtn.addEventListener("click", function () {
			window.print();
		});

	const toggle = document.getElementById("details-toggle");
	if (toggle) {
		toggle.addEventListener("change", toggleDetails);
	}

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
