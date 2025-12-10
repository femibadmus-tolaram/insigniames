/** @format */

let currentUserRole = null;

document.addEventListener("DOMContentLoaded", async function () {
	document.body.style.zoom = "90%";
	await loadUserRole();
	transformNavBasedOnRole();
	highlightActiveLink();
});

async function loadUserRole() {
	try {
		const userResponse = await fetch("/api/users/me");
		if (!userResponse.ok) throw new Error("Failed to load user data");
		const userData = await userResponse.json();

		const rolesResponse = await fetch("/api/roles");
		if (!rolesResponse.ok) throw new Error("Failed to load roles");
		const roles = await rolesResponse.json();

		const userRole = roles.find((r) => r.id === userData.role_id);
		currentUserRole = userRole ? userRole.name : null;
	} catch (error) {
		console.error("Error loading role:", error);
	}
}

function transformNavBasedOnRole() {
	const nav = document.querySelector("#main-nav");
	if (!nav) return;

	const roleNavMap = {
		Dashboard: ["Dashboard", "Jobs", "Rolls"],
		Production: ["Production", "Downtime", "Scrap", "Actual Consumable"],
		Settings: ["Settings", "Materials", "Machines", "Sections", "Manage Lookups", "Users Management", "Roles & Permissions", "Logout"],
	};

	const defaultNav = `
        <li class="relative group">
            <a href="/" class="px-3 py-2 rounded-lg hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2"
                ><i class="fas fa-chart-simple"></i> Dashboard <i class="fas fa-chevron-down text-xs ml-1"></i
            ></a>
            <div class="absolute left-0 top-full mt-1 w-48 bg-white shadow-lg rounded-lg border border-gray-200 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-50">
                <div class="py-2">
                    <a href="/jobs" class="px-4 py-2 hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2 text-sm">
                        <i class="fas fa-briefcase"></i> Jobs
                    </a>
                    <a href="/rolls" class="px-4 py-2 hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2 text-sm">
                        <i class="fas fa-layer-group"></i> Rolls
                    </a>
                </div>
            </div>
        </li>

        <li class="relative group">
            <a href="/production" class="px-3 py-2 rounded-lg hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2"
                ><i class="fas fa-industry"></i> Production <i class="fas fa-chevron-down text-xs ml-1"></i
            ></a>
            <div class="absolute left-0 top-full mt-1 w-48 bg-white shadow-lg rounded-lg border border-gray-200 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-50">
                <div class="py-2">
                    <a href="/downtime" class="px-4 py-2 hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2 text-sm">
                        <i class="fas fa-clock"></i> Downtime
                    </a>
                    <a href="/scrap" class="px-4 py-2 hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2 text-sm">
                        <i class="fas fa-trash"></i> Scrap
                    </a>
                    <a href="/consumables" class="px-4 py-2 hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2 text-sm">
                        <i class="fas fa-flask"></i> Actual Consumable
                    </a>
                </div>
            </div>
        </li>

        <li class="relative group">
            <a href="/settings" class="px-3 py-2 rounded-lg hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2"
                ><i class="fas fa-cog"></i> Settings <i class="fas fa-chevron-down text-xs ml-1"></i
            ></a>
            <div class="absolute left-0 top-full mt-1 w-48 bg-white shadow-lg rounded-lg border border-gray-200 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-50">
                <div class="py-2">
                    <a href="/materials" class="px-4 py-2 hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2 text-sm">
                        <i class="fas fa-industry"></i> Materials
                    </a>
                    <a href="/machines" class="px-4 py-2 hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2 text-sm">
                        <i class="fas fa-cogs"></i> Machines
                    </a>
                    <a href="/sections" class="px-4 py-2 hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2 text-sm">
                        <i class="fas fa-building"></i> Sections
                    </a>
                    <a href="/lookups" class="px-4 py-2 hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2 text-sm">
                        <i class="fas fa-list"></i> Manage Lookups
                    </a>
                    <div class="border-t border-gray-200 my-1"></div>
                    <a href="/users" class="px-4 py-2 hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2 text-sm">
                        <i class="fas fa-users"></i> Users Management
                    </a>
                    <a href="/roles" class="px-4 py-2 hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2 text-sm">
                        <i class="fas fa-user-tag"></i> Roles & Permissions
                    </a>
                    <div class="border-t border-gray-200 my-1"></div>
                    <a href="/logout" class="px-4 py-2 hover:bg-red-50 hover:text-red-600 flex items-center gap-2 text-sm">
                        <i class="fas fa-sign-out-alt"></i> Logout
                    </a>
                </div>
            </div>
        </li>
    `;

	let roleFound = false;
	for (const [dropdownName, dropdownItems] of Object.entries(roleNavMap)) {
		if (dropdownItems.some((item) => item === currentUserRole)) {
			nav.innerHTML = "";
			dropdownItems.forEach((item) => {
				const navItemHTML = createNavItemHTML(item);
				nav.innerHTML += navItemHTML;
			});
			roleFound = true;
			break;
		}
	}

	if (!roleFound) {
		nav.innerHTML = defaultNav;
	}
}

function createNavItemHTML(itemName) {
	const hrefMap = {
		Dashboard: "/",
		Jobs: "/jobs",
		Rolls: "/rolls",
		Production: "/production",
		Downtime: "/downtime",
		Scrap: "/scrap",
		"Actual Consumable": "/consumables",
		Settings: "/settings",
		Materials: "/materials",
		Machines: "/machines",
		Sections: "/sections",
		"Manage Lookups": "/lookups",
		"Users Management": "/users",
		"Roles & Permissions": "/roles",
		Logout: "/logout",
	};

	const iconMap = {
		Dashboard: "fa-chart-simple",
		Jobs: "fa-briefcase",
		Rolls: "fa-layer-group",
		Production: "fa-industry",
		Downtime: "fa-clock",
		Scrap: "fa-trash",
		"Actual Consumable": "fa-flask",
		Settings: "fa-cog",
		Materials: "fa-industry",
		Machines: "fa-cogs",
		Sections: "fa-building",
		"Manage Lookups": "fa-list",
		"Users Management": "fa-users",
		"Roles & Permissions": "fa-user-tag",
		Logout: "fa-sign-out-alt",
	};

	return `
        <li>
            <a href="${hrefMap[itemName]}" class="px-3 py-2 rounded-lg hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2">
                <i class="fas ${iconMap[itemName]}"></i> ${itemName}
            </a>
        </li>
    `;
}

function highlightActiveLink() {
	const currentPath = window.location.pathname;
	const navLinks = document.querySelectorAll("#main-nav a[href]");

	navLinks.forEach((link) => {
		if (link.getAttribute("href") === currentPath) {
			link.classList.add("bg-blue-50", "text-blue-600");
		}
	});
}

function truncateText(text, maxLength) {
	if (!text) {
		return "N/A";
	}
	if (text.length > maxLength) {
		return text.substring(0, maxLength) + "...";
	}
	return text;
}

function showNotification(message, type = "info") {
	const existingNotification = document.querySelector(".notification");
	if (existingNotification) {
		existingNotification.remove();
	}

	if (
		message.toLowerCase().includes("failed to fetch") ||
		message.toLowerCase().includes("aborterror") ||
		message.toLowerCase().includes("aborted")
	) {
		message = "No or slow internet, refresh or try again";
	}

	const notification = document.createElement("div");
	notification.className = `notification fixed top-4 right-4 px-4 py-3 rounded-lg shadow-lg z-50 ${
		type === "success"
			? "bg-green-500 text-white"
			: type === "error"
			? "bg-red-500 text-white"
			: type === "warning"
			? "bg-red-500 text-white"
			: "bg-blue-500 text-white"
	}`;
	notification.textContent = message;

	document.body.appendChild(notification);

	setTimeout(() => {
		notification.remove();
	}, 5000);
}

function formatDate(dateString) {
	const date = new Date(dateString);
	return date.toLocaleDateString();
}

function formatDateTime(dateString) {
	const date = new Date(dateString);
	return date.toLocaleString();
}

function formatDateTimeLocal(dateString) {
	const date = new Date(dateString);
	return date.toISOString().slice(0, 16);
}

function saveAsExcel(buffer, filename) {
	const blob = new Blob([buffer], { type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" });
	const url = URL.createObjectURL(blob);
	const a = document.createElement("a");
	a.href = url;
	a.download = filename;
	document.body.appendChild(a);
	a.click();
	setTimeout(() => {
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}, 100);
}

function escapeHtml(text) {
	const div = document.createElement("div");
	div.textContent = text;
	return div.innerHTML;
}

function updatePerPageOptions(totalCount) {
	const perPageSelect = document.getElementById("per-page");
	perPageSelect.innerHTML = "";

	let options = [];
	const maxOptions = 5;
	const step = Math.ceil(totalCount / maxOptions);

	for (let i = 1; i <= maxOptions; i++) {
		const value = Math.min(i * step, totalCount);
		if (!options.includes(value)) {
			options.push(value);
		}
	}

	if (options.length === 0) options.push(10);

	options.forEach((option) => {
		const opt = document.createElement("option");
		opt.value = option;
		opt.textContent = option === totalCount ? "All" : option;
		if (option === itemsPerPage) {
			opt.selected = true;
		}
		perPageSelect.appendChild(opt);
	});
}

function showLoading(show, table_id) {
	const loadingMessage = document.getElementById("loading-message");
	const table = document.getElementById(table_id);

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

function formatMinutes(minutes) {
	if (minutes < 60) {
		return `${minutes}m`;
	} else {
		const hours = Math.floor(minutes / 60);
		const mins = minutes % 60;
		return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
	}
}

function formatDbError(errorMessage) {
	const errorMap = {
		// Machines
		"UNIQUE constraint failed: machines.name": "Routing Name already exists",
		"UNIQUE constraint failed: machines.label": "Routing Code already exists",

		// Users
		"UNIQUE constraint failed: users.staffid": "Staff ID already exists",
		"UNIQUE constraint failed: users.phone_number": "Phone number already exists",

		// Shifts
		"UNIQUE constraint failed: shifts.name": "Shift name already exists",

		// Colours
		"UNIQUE constraint failed: colours.name": "Colour name already exists",

		// Solvent Types
		"UNIQUE constraint failed: solvent_types.name": "Solvent type name already exists",

		// Scrap Types
		"UNIQUE constraint failed: scrap_types.name": "Scrap type name already exists",

		// Downtime Reasons
		"UNIQUE constraint failed: downtime_reasons.name": "Downtime reason name already exists",

		// Flag Reasons
		"UNIQUE constraint failed: flag_reasons.name": "Flag reason name already exists",

		// Roles
		"UNIQUE constraint failed: roles.name": "Role name already exists",

		// Permissions
		"UNIQUE constraint failed: permissions.codename": "Permission codename already exists",

		// Rolls
		"UNIQUE constraint failed: rolls.output_roll_no": "Output roll number already exists",

		// Jobs
		"UNIQUE constraint failed: jobs.batch_roll_no": "Batch roll number already exists",

		// Content Type
		"UNIQUE constraint failed: content_type.model": "Content type model already exists",

		// User Machines
		"UNIQUE constraint failed: user_machines.user_id, user_machines.machine_id": "User is already assigned to this machine",

		// Role Permissions
		"UNIQUE constraint failed: role_permissions.role_id, role_permissions.permission_id": "Permission is already assigned to this role",

		// Foreign Key Errors
		"FOREIGN KEY constraint failed": "Related record not found",
		"FOREIGN KEY constraint failed: users.role_id": "Role not found",
		"FOREIGN KEY constraint failed: jobs.shift_id": "Shift not found",
		"FOREIGN KEY constraint failed: jobs.machine_id": "Machine not found",
		"FOREIGN KEY constraint failed: jobs.created_by": "User not found",
		"FOREIGN KEY constraint failed: rolls.job_id": "Job not found",
		"FOREIGN KEY constraint failed: rolls.flag_reason_id": "Flag reason not found",
		"FOREIGN KEY constraint failed: rolls.created_by": "User not found",
		"FOREIGN constraint failed: downtimes.shift_id": "Shift not found",
		"FOREIGN KEY constraint failed: downtimes.downtime_reason_id": "Downtime reason not found",
		"FOREIGN KEY constraint failed: downtimes.created_by": "User not found",
		"FOREIGN KEY constraint failed: scraps.shift_id": "Shift not found",
		"FOREIGN KEY constraint failed: scraps.scrap_type_id": "Scrap type not found",
		"FOREIGN KEY constraint failed: scraps.created_by": "User not found",
		"FOREIGN KEY constraint failed: ink_usages.shift_id": "Shift not found",
		"FOREIGN KEY constraint failed: ink_usages.colour_id": "Colour not found",
		"FOREIGN KEY constraint failed: ink_usages.created_by": "User not found",
		"FOREIGN KEY constraint failed: solvent_usages.shift_id": "Shift not found",
		"FOREIGN KEY constraint failed: solvent_usages.solvent_type_id": "Solvent type not found",
		"FOREIGN KEY constraint failed: solvent_usages.created_by": "User not found",
		"FOREIGN KEY constraint failed: user_machines.user_id": "User not found",
		"FOREIGN KEY constraint failed: user_machines.machine_id": "Machine not found",
		"FOREIGN KEY constraint failed: role_permissions.role_id": "Role not found",
		"FOREIGN KEY constraint failed: role_permissions.permission_id": "Permission not found",
		"FOREIGN KEY constraint failed: permissions.content_type_id": "Content type not found",

		// NOT NULL Constraints
		"NOT NULL constraint failed: users.full_name": "Full name is required",
		"NOT NULL constraint failed: users.staffid": "Staff ID is required",
		"NOT NULL constraint failed: users.password": "Password is required",
		"NOT NULL constraint failed: users.status": "Status is required",
		"NOT NULL constraint failed: users.role_id": "Role is required",
		"NOT NULL constraint failed: jobs.shift_id": "Shift is required",
		"NOT NULL constraint failed: jobs.production_order": "Production order is required",
		"NOT NULL constraint failed: jobs.batch_roll_no": "Batch roll number is required",
		"NOT NULL constraint failed: jobs.start_weight": "Start weight is required",
		"NOT NULL constraint failed: jobs.start_meter": "Start meter is required",
		"NOT NULL constraint failed: rolls.output_roll_no": "Output roll number is required",
		"NOT NULL constraint failed: rolls.final_meter": "Final meter is required",
		"NOT NULL constraint failed: rolls.final_weight": "Final weight is required",
		"NOT NULL constraint failed: downtimes.shift_id": "Shift is required",
		"NOT NULL constraint failed: downtimes.start_time": "Start time is required",
		"NOT NULL constraint failed: downtimes.end_time": "End time is required",
		"NOT NULL constraint failed: downtimes.duration_minutes": "Duration is required",
		"NOT NULL constraint failed: downtimes.downtime_reason_id": "Downtime reason is required",
		"NOT NULL constraint failed: scraps.shift_id": "Shift is required",
		"NOT NULL constraint failed: scraps.time": "Time is required",
		"NOT NULL constraint failed: scraps.scrap_type_id": "Scrap type is required",
		"NOT NULL constraint failed: scraps.weight_kg": "Weight is required",
		"NOT NULL constraint failed: ink_usages.shift_id": "Shift is required",
		"NOT NULL constraint failed: ink_usages.colour_id": "Colour is required",
		"NOT NULL constraint failed: ink_usages.batch_code": "Batch code is required",
		"NOT NULL constraint failed: ink_usages.kgs_issued": "Kgs issued is required",
		"NOT NULL constraint failed: solvent_usages.shift_id": "Shift is required",
		"NOT NULL constraint failed: solvent_usages.solvent_type_id": "Solvent type is required",
		"NOT NULL constraint failed: solvent_usages.kgs_issued": "Kgs issued is required",

		// Other Common Errors
		"no such table": "Database table not found",
		"record not found": "Record not found",
		"not found": "Record not found",
		"cannot delete": "Cannot delete this record",
		"database is locked": "Database is busy, please try again",
		"disk I/O error": "Storage error occurred",
	};

	for (const [key, value] of Object.entries(errorMap)) {
		if (errorMessage.includes(key)) {
			return value;
		}
	}

	return errorMessage;
}

async function handleApiResponse(response) {
	const text = await response.text();
	if (response.ok) {
		try {
			return JSON.parse(text);
		} catch {
			return text;
		}
	} else {
		let errorText;
		try {
			const errorData = JSON.parse(text);
			errorText = errorData.message || errorData.error || text;
		} catch {
			errorText = text;
		}
		throw new Error(formatDbError(errorText));
	}
}

function populateSelect(id, items, key, defaultText) {
	const sel = document.getElementById(id);
	sel.innerHTML = `<option value="">${defaultText}</option>`;
	items.forEach((i) => {
		const opt = document.createElement("option");
		opt.value = i.id;
		opt.textContent = i[key];
		sel.appendChild(opt);
	});
}

function setupShiftTimeRestrictions() {
	const shiftSelect = document.getElementById("shift");
	const timeInput = document.getElementById("time");
	const startTimeInput = document.getElementById("start-time");
	const endTimeInput = document.getElementById("end-time");

	const timeInputs = [];
	if (timeInput) timeInputs.push(timeInput);
	if (startTimeInput) timeInputs.push(startTimeInput);
	if (endTimeInput) timeInputs.push(endTimeInput);

	timeInputs.forEach((input) => {
		input.addEventListener("input", function () {
			const shiftId = shiftSelect.value;
			const selectedTime = new Date(this.value);
			const hours = selectedTime.getHours();

			if (shiftId === "1" && (hours < 7 || hours >= 19)) {
				this.value = "";
				showNotification("Morning shift allows time between 7:00 AM and 7:00 PM only", "error");
			} else if (shiftId === "2" && hours >= 7 && hours < 19) {
				this.value = "";
				showNotification("Night shift allows time between 7:00 PM and 7:00 AM only", "error");
			}
		});
	});

	shiftSelect.addEventListener("change", function () {
		updateTimeRestrictions(this.value);
	});
}

function updateTimeRestrictions(shiftId) {
	const timeInput = document.getElementById("time");
	const startTimeInput = document.getElementById("start-time");
	const endTimeInput = document.getElementById("end-time");

	const timeInputs = [];
	if (timeInput) timeInputs.push(timeInput);
	if (startTimeInput) timeInputs.push(startTimeInput);
	if (endTimeInput) timeInputs.push(endTimeInput);

	const now = new Date();
	const minDate = new Date(now);
	minDate.setDate(minDate.getDate() - 3);
	const maxDate = new Date(now);
	maxDate.setDate(maxDate.getDate() + 3);

	timeInputs.forEach((input) => {
		if (shiftId === "1") {
			input.min = minDate.toISOString().slice(0, 10) + "T07:00";
			input.max = maxDate.toISOString().slice(0, 10) + "T19:00";
			input.title = "Morning shift: 7:00 AM to 7:00 PM";
		} else if (shiftId === "2") {
			input.min = minDate.toISOString().slice(0, 10) + "T19:00";
			input.max = maxDate.toISOString().slice(0, 10) + "T07:00";
			input.title = "Night shift: 7:00 PM to 7:00 AM next day";
		} else {
			input.removeAttribute("min");
			input.removeAttribute("max");
			input.removeAttribute("title");
		}
	});
}

function autoSelectShift() {
	const shiftSelects = [
		document.getElementById("shift"),
		document.getElementById("current-shift"),
		document.getElementById("ink-shift"),
		document.getElementById("solvent-shift"),
	];

	const now = new Date();
	const hours = now.getHours();
	const shiftValue = hours >= 7 && hours < 19 ? "1" : "2";

	shiftSelects.forEach((select) => {
		if (select) {
			select.value = shiftValue;
			updateTimeRestrictions(shiftValue);
		}
	});
}

function formatWeight(kg) {
	const units = ["kg", "t", "kt", "Mt", "Gt"];
	let i = 0;
	while (kg >= 1000 && i < units.length - 1) {
		kg /= 1000;
		i++;
	}
	return `${kg.toFixed(2)} ${units[i]}`;
}

function formatMeter(m) {
	const units = ["m", "km", "Mm", "Gm", "Tm"];
	let i = 0;
	while (m >= 1000 && i < units.length - 1) {
		m /= 1000;
		i++;
	}
	return `${m.toFixed(2)} ${units[i]}`;
}

function formatTime(dateTimeString) {
	if (!dateTimeString) return "-";

	const date = new Date(dateTimeString);
	if (isNaN(date.getTime())) return "-";

	const day = date.getDate().toString().padStart(2, "0");
	const month = (date.getMonth() + 1).toString().padStart(2, "0");
	const year = date.getFullYear();
	const hours = date.getHours().toString().padStart(2, "0");
	const minutes = date.getMinutes().toString().padStart(2, "0");

	return `${day}/${month}/${year} ${hours}:${minutes}`;
}

function formatDowntime(minutes) {
	if (!minutes || minutes === 0) return "0 min";

	if (minutes < 60) {
		return `${minutes} min`;
	} else {
		const hours = Math.floor(minutes / 60);
		const remainingMinutes = minutes % 60;
		return remainingMinutes > 0 ? `${hours}h ${remainingMinutes}m` : `${hours}h`;
	}
}
