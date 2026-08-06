document.addEventListener('DOMContentLoaded', () => {
    const navItems = document.querySelectorAll('.nav-item');
    const pageTitle = document.getElementById('page-title');
    const contentArea = document.getElementById('content-area');
    const modalContainer = document.getElementById('modal-container');

    const routes = {
        dashboard: renderDashboard,
        extensions: renderExtensions,
        trunks: renderTrunks,
        dialplan: renderDialplan,
        system: renderSystem
    };

    function navigateTo(page) {
        navItems.forEach(item => {
            item.classList.toggle('active', item.dataset.page === page);
        });

        pageTitle.textContent = page.charAt(0).toUpperCase() + page.slice(1);
        if (routes[page]) routes[page]();
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
            const res = await fetch('/api/v1/system/dashboard');
            const data = await res.json();
            
            contentArea.innerHTML = `
                <div class="grid-cards">
                    <div class="card">
                        <h3>Registered Subscribers</h3>
                        <div class="metric">${data.total_registered_subscribers}</div>
                    </div>
                    <div class="card">
                        <h3>Active Calls</h3>
                        <div class="metric">${data.active_calls}</div>
                    </div>
                    <div class="card">
                        <h3>Database Health</h3>
                        <div style="font-size: 1.1rem; font-weight: bold; color: var(--success); margin-top: 0.5rem;">${data.database_status}</div>
                    </div>
                </div>

                <h2 style="margin-top: 2rem; margin-bottom: 1rem;">System Services Status</h2>
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>Service Binary</th>
                            <th>Port / Transport</th>
                            <th>Status</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${data.services.map(svc => `
                            <tr>
                                <td><strong>${svc.name}</strong></td>
                                <td><code>${svc.port}</code></td>
                                <td>
                                    <span class="badge ${svc.status === 'online' ? '' : 'btn-danger'}" style="${svc.status === 'standby' ? 'background: rgba(245, 158, 11, 0.2); color: var(--warning); border: 1px solid var(--warning);' : ''}">
                                        ${svc.status === 'online' ? '🟢 Running' : (svc.status === 'standby' ? '🟡 Standby' : '🔴 Stopped')}
                                    </span>
                                </td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            `;
        } catch (e) {
            contentArea.innerHTML = `<div class="card">Error loading dashboard metrics</div>`;
        }
    }

    async function renderExtensions() {
        try {
            const res = await fetch('/api/v1/extensions');
            const extensions = await res.json();
            
            contentArea.innerHTML = `
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;">
                    <h2>Extension Management</h2>
                    <button id="btn-add-ext" class="btn">+ Add Extension</button>
                </div>
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>Extension</th>
                            <th>Name</th>
                            <th>Email</th>
                            <th>SIP Status</th>
                            <th>Recording</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${extensions.map(ext => `
                            <tr>
                                <td><strong>${ext.extension_number}</strong></td>
                                <td>${ext.display_name}</td>
                                <td>${ext.email || '-'}</td>
                                <td>
                                    <span class="badge ${ext.is_registered ? '' : 'btn-danger'}" style="${!ext.is_registered ? 'background: rgba(239, 68, 68, 0.2); color: #ef4444; border: 1px solid #ef4444;' : ''}">
                                        ${ext.is_registered ? '🟢 Registered' : '🔴 Offline'}
                                    </span>
                                </td>
                                <td>${ext.record_calls == 1 ? '🔴 Enabled' : '⚪ Disabled'}</td>
                                <td>
                                    <button class="btn btn-sm btn-edit" data-ext='${JSON.stringify(ext).replace(/'/g, "&apos;")}'>✏️ Edit</button>
                                    <button class="btn btn-sm btn-danger btn-delete" data-id="${ext.id}" data-number="${ext.extension_number}">🗑️ Delete</button>
                                </td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            `;

            document.getElementById('btn-add-ext').addEventListener('click', () => openExtensionModal());

            document.querySelectorAll('.btn-edit').forEach(btn => {
                btn.addEventListener('click', () => {
                    const ext = JSON.parse(btn.dataset.ext);
                    openExtensionModal(ext);
                });
            });

            document.querySelectorAll('.btn-delete').forEach(btn => {
                btn.addEventListener('click', async () => {
                    if (confirm(`Are you sure you want to delete Extension ${btn.dataset.number}?`)) {
                        await fetch(`/api/v1/extensions/${btn.dataset.id}`, { method: 'DELETE' });
                        renderExtensions();
                    }
                });
            });
        } catch (e) {
            contentArea.innerHTML = `<div class="card">Error loading extension data</div>`;
        }
    }

    function openExtensionModal(ext = null) {
        const isEdit = !!ext;
        modalContainer.innerHTML = `
            <div class="modal-backdrop" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                <div class="modal" style="background: var(--surface); padding: 2rem; border-radius: 12px; border: 1px solid var(--border); max-width: 600px; width: 90%; max-height: 90vh; overflow-y: auto;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;">
                        <h2>${isEdit ? 'Edit Extension ' + ext.extension_number : 'Add New Extension'}</h2>
                        <button id="closeModal" style="background: none; border: none; color: var(--text-muted); font-size: 1.5rem; cursor: pointer;">&times;</button>
                    </div>
                    <form id="extForm">
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem;">
                            <div class="form-group">
                                <label>Extension Number *</label>
                                <input type="text" id="extNum" class="form-control" value="${ext ? ext.extension_number : ''}" ${isEdit ? 'disabled' : 'required'} placeholder="e.g. 101">
                            </div>
                            <div class="form-group">
                                <label>Password ${isEdit ? '(Optional)' : '*'}</label>
                                <input type="password" id="extPass" class="form-control" ${!isEdit ? 'required' : ''} placeholder="${isEdit ? 'Leave blank to keep unchanged' : 'SIP Password'}">
                            </div>
                        </div>

                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem;">
                            <div class="form-group">
                                <label>Display Name *</label>
                                <input type="text" id="extName" class="form-control" value="${ext ? ext.display_name : ''}" required placeholder="e.g. Alice Smith">
                            </div>
                            <div class="form-group">
                                <label>Email Address</label>
                                <input type="email" id="extEmail" class="form-control" value="${ext && ext.email ? ext.email : ''}" placeholder="user@company.com">
                            </div>
                        </div>

                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem;">
                            <div class="form-group">
                                <label>NAT Mode</label>
                                <select id="extNat" class="form-control">
                                    <option value="auto" ${ext && ext.nat_mode === 'auto' ? 'selected' : ''}>Auto (Detect rport)</option>
                                    <option value="force_rport" ${ext && ext.nat_mode === 'force_rport' ? 'selected' : ''}>Force rport</option>
                                    <option value="stun" ${ext && ext.nat_mode === 'stun' ? 'selected' : ''}>STUN Traversal</option>
                                    <option value="disabled" ${ext && ext.nat_mode === 'disabled' ? 'selected' : ''}>Disabled (LAN only)</option>
                                </select>
                            </div>
                            <div class="form-group">
                                <label>OPTIONS Ping (sec)</label>
                                <input type="number" id="extQualify" class="form-control" value="${ext ? ext.qualify_frequency : 60}" min="0" max="3600">
                            </div>
                        </div>

                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem;">
                            <div class="form-group">
                                <label>Min Expires (sec)</label>
                                <input type="number" id="extMinExpires" class="form-control" value="${ext ? ext.min_expires : 60}" min="10">
                            </div>
                            <div class="form-group">
                                <label>Max Expires (sec)</label>
                                <input type="number" id="extMaxExpires" class="form-control" value="${ext ? ext.max_expires : 3600}" max="86400">
                            </div>
                        </div>

                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem;">
                            <div class="form-group">
                                <label>Max Concurrent Logins</label>
                                <input type="number" id="extMaxLogins" class="form-control" value="${ext ? ext.max_concurrent_logins : 1}" min="1" max="10">
                            </div>
                            <div class="form-group">
                                <label>Allowed Transports</label>
                                <input type="text" id="extTransports" class="form-control" value="${ext ? ext.allowed_transport : 'udp,tcp,tls,ws'}">
                            </div>
                        </div>

                        <div style="display: flex; gap: 1.5rem; margin-top: 1rem;">
                            <div class="form-group" style="display: flex; align-items: center; gap: 0.5rem;">
                                <input type="checkbox" id="extRecord" ${ext && ext.record_calls == 1 ? 'checked' : ''} style="width: auto;">
                                <label for="extRecord" style="margin: 0;">Record Calls</label>
                            </div>
                            <div class="form-group" style="display: flex; align-items: center; gap: 0.5rem;">
                                <input type="checkbox" id="extAuth" ${!ext || ext.auth_required == 1 ? 'checked' : ''} style="width: auto;">
                                <label for="extAuth" style="margin: 0;">Require Digest Auth</label>
                            </div>
                        </div>

                        <div style="display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 1.5rem;">
                            <button type="button" class="btn" id="cancelModal" style="background: var(--surface-light);">Cancel</button>
                            <button type="submit" class="btn btn-primary" id="saveExtBtn">${isEdit ? 'Update Extension' : 'Create Extension'}</button>
                        </div>
                    </form>
                </div>
            </div>
        `;
        modalContainer.style.display = 'block';

        const closeModal = () => { modalContainer.style.display = 'none'; };
        document.getElementById('closeModal').addEventListener('click', closeModal);
        document.getElementById('cancelModal').addEventListener('click', closeModal);

        document.getElementById('extForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            const payload = {
                display_name: document.getElementById('extName').value,
                email: document.getElementById('extEmail').value || null,
                record_calls: document.getElementById('extRecord').checked,
                nat_mode: document.getElementById('extNat').value,
                qualify_frequency: parseInt(document.getElementById('extQualify').value) || 60,
                min_expires: parseInt(document.getElementById('extMinExpires').value) || 60,
                max_expires: parseInt(document.getElementById('extMaxExpires').value) || 3600,
                max_concurrent_logins: parseInt(document.getElementById('extMaxLogins').value) || 1,
                allowed_transport: document.getElementById('extTransports').value || 'udp,tcp,tls,ws',
                auth_required: document.getElementById('extAuth').checked
            };
            
            const pass = document.getElementById('extPass').value;
            if (pass) payload.password = pass;

            if (isEdit) {
                await fetch(`/api/v1/extensions/${ext.id}`, {
                    method: 'PUT',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(payload)
                });
            } else {
                payload.extension_number = document.getElementById('extNum').value;
                await fetch('/api/v1/extensions', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(payload)
                });
            }

            modalContainer.style.display = 'none';
            renderExtensions();
        });
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
