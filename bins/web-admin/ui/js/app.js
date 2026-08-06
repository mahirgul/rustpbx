document.addEventListener('DOMContentLoaded', () => {
    const navItems = document.querySelectorAll('.nav-item');
    const pageTitle = document.getElementById('page-title');
    const contentArea = document.getElementById('content-area');

    const routes = {
        dashboard: renderDashboard,
        extensions: renderExtensions,
        trunks: renderTrunks,
        dialplan: renderDialplan,
        system: renderSystem
    };

    function navigateTo(page) {
        navItems.forEach(item => {
            if (item.dataset.page === page) {
                item.classList.add('active');
            } else {
                item.classList.remove('active');
            }
        });

        pageTitle.textContent = page.charAt(0).toUpperCase() + page.slice(1);
        if (routes[page]) {
            routes[page]();
        }
    }

    navItems.forEach(item => {
        item.addEventListener('click', (e) => {
            e.preventDefault();
            const page = item.dataset.page;
            window.location.hash = page;
            navigateTo(page);
        });
    });

    const initialPage = window.location.hash.replace('#', '') || 'dashboard';
    navigateTo(initialPage);

    async function renderDashboard() {
        try {
            const res = await fetch('/api/v1/extensions');
            const extensions = await res.json();
            
            contentArea.innerHTML = `
                <div class="grid-cards">
                    <div class="card">
                        <h3>Active Extensions</h3>
                        <div class="metric">${extensions.length}</div>
                    </div>
                    <div class="card">
                        <h3>Active Calls</h3>
                        <div class="metric">0</div>
                    </div>
                    <div class="card">
                        <h3>SIP Trunks</h3>
                        <div class="metric">1</div>
                    </div>
                </div>
                <h2>Registered Telephony Devices</h2>
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>Extension</th>
                            <th>Name</th>
                            <th>Email</th>
                            <th>Recording</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${extensions.map(ext => `
                            <tr>
                                <td><strong>${ext.extension_number}</strong></td>
                                <td>${ext.display_name}</td>
                                <td>${ext.email || '-'}</td>
                                <td>${ext.record_calls == 1 ? '🔴 Enabled' : '⚪ Disabled'}</td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            `;
        } catch (e) {
            contentArea.innerHTML = `<div class="card">Error loading dashboard data</div>`;
        }
    }

    async function renderExtensions() {
        contentArea.innerHTML = `<div class="card"><h2>Extension Management</h2><p>Manage SIP subscribers and extension credentials.</p></div>`;
    }

    async function renderTrunks() {
        contentArea.innerHTML = `<div class="card"><h2>SIP Trunks</h2><p>Manage outbound/inbound PSTN gateways.</p></div>`;
    }

    async function renderDialplan() {
        contentArea.innerHTML = `<div class="card"><h2>Dialplan Rules</h2><p>Call routing engine and pattern matching.</p></div>`;
    }

    async function renderSystem() {
        contentArea.innerHTML = `<div class="card"><h2>System Info</h2><p>RustPBX Embedded Node Configuration.</p></div>`;
    }
});
