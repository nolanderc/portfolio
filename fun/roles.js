const div_config = document.getElementById("config");
const div_roles = document.getElementById("roles");
const div_total = document.getElementById("total");
const button_add_role = document.getElementById("add_role");
const button_start = document.getElementById("start");
const button_reset = document.getElementById("reset");

const div_assignment = document.getElementById("assignment");
div_assignment.style.display = "none";
const button_next = document.getElementById("next");
const div_display = document.getElementById("display");
let assignment = [];
let assignment_next = 0
let assignment_state = 'reveal';

function reset() {
    div_roles.innerHTML = ''
    div_roles.appendChild(createRole("Civilian", 2));
    div_roles.appendChild(createRole("Doctor", 1));
    div_roles.appendChild(createRole("Sheriff", 1));
    div_roles.appendChild(createRole("Mafia", 2));
    saveRoles();
}

button_reset.onclick = () => {
    reset()
}

function getRoles() {
    const role_elements = div_roles.getElementsByClassName("role");
    const roles = [];
    for (const element of role_elements) {
        const inputs = element.getElementsByTagName("input");
        roles.push({
            name: inputs[0].value,
            count: inputs[1].valueAsNumber,
        });
    }
    return roles;
}

function updateTotal() {
    let count = 0;
    getRoles().forEach((role) => count += role.count)
    div_total.innerHTML = `Players: ${count}`;
}

function saveRoles() {
    const roles = getRoles();
    localStorage.setItem("fun.roles", JSON.stringify(roles));
    updateTotal()
}

function loadRoles() {
    const roles = localStorage.getItem("fun.roles");
    return roles == null ? null : JSON.parse(roles);
}

function createRole(name, count) {
    const role = document.createElement("p");
    role.innerHTML = `
        <div class="role">
            <button class="delete">Delete</button>
            <input type="text" value="${name}" />
            <input
                type="number"
                min="0"
                value="${count}"
                inputmode="numeric"
                pattern="[0-9]*"
            />
        </div>
    `;
    const delete_button = role.getElementsByClassName("delete")[0];
    delete_button.onclick = () => {
        role.remove();
        saveRoles();
    };
    const inputs = role.getElementsByTagName("input");
    for (const input of inputs) {
        input.onchange = () => saveRoles();
    }
    return role;
}

button_add_role.onclick = () => {
    div_roles.appendChild(createRole("", 1));
    saveRoles();
};

const old_roles = loadRoles();
if (old_roles) {
    for (const role of old_roles) {
        div_roles.appendChild(createRole(role.name, role.count));
    }
} else {
    reset()
}
updateTotal()

function shuffle(array) {
    let index = array.length;
    while (index > 0) {
        const target = Math.floor(Math.random() * index)
        index--;
        const tmp = array[index]
        array[index] = array[target]
        array[target] = tmp
    }
}

button_start.onclick = () => {
    saveRoles()

    div_config.style.display = "none";
    div_assignment.style.display = "flex";

    assignment = [];
    assignment_next = 0;

    for (const role of getRoles()) {
        for (let i = 0; i < role.count; i++) {
            assignment.push(role.name);
        }
    }
    shuffle(assignment)

    button_next.innerText = `Reveal Player ${assignment_next + 1}`
    assignment_state = 'reveal';
};

button_next.onclick = () => {
    if (assignment_state == 'reveal') {
        div_display.innerText = assignment[assignment_next]

        button_next.disabled = true;
        setTimeout(() => {
            button_next.disabled = false;
            assignment_state = 'next';
            button_next.innerText = 'Next';
        }, 1000)
    } else if (assignment_state == 'next') {
        div_display.innerText = ''
        assignment_next++;

        button_next.disabled = true;
        setTimeout(() => {
            button_next.disabled = false;
            if (assignment_next < assignment.length) {
                assignment_state = 'reveal';
                button_next.innerText = `Reveal Player ${assignment_next + 1}`;
            } else {
                assignment_state = 'finish';
                button_next.innerText = 'Finish';
            }
        }, 1000)
    } else {
        div_config.style.display = 'flex'
        div_assignment.style.display = 'none'
    }
}
