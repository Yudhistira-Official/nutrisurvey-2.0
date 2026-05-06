let currentSessionFoods = [];
let activeMealTimes = [
    { id: 'BREAKFAST', label: 'Makan Pagi' },
    { id: 'LUNCH', label: 'Makan Siang' },
    { id: 'DINNER', label: 'Makan Malam' }
];

let targets = { kcal: 0, carbs: 0, protein: 0, fat: 0 };
let nutrientMetadata = [];
let filters = [];

function generateMealId() {
    return `MEAL_${Date.now()}_${Math.floor(Math.random() * 100000)}`;
}

// Robust key mapping for macros
const MACRO_MAP = {
    energy: ['energi', 'energy', 'energy (kcal)', 'energi (kkal)', 'energi total', 'total energy'],
    carbs: ['karbohidrat total', 'karbohidrat', 'carbohydrate', 'carbohydrates', 'total carbohydrate', 'karbo'],
    protein: ['protein', 'total protein'],
    fat: ['lemak total', 'lemak', 'fat', 'total fat', 'fats']
};

function toNumber(val) {
    if (val === undefined || val === null) return 0;
    if (typeof val === 'number') return Number.isFinite(val) ? val : 0;
    const normalized = String(val).trim().replace(',', '.');
    const n = parseFloat(normalized);
    return Number.isFinite(n) ? n : 0;
}

function getMacroValue(nutrients, type) {
    if (!nutrients) return 0;
    const keys = MACRO_MAP[type] || [];
    for (const k of keys) {
        const val = nutrients[k] || nutrients[k.charAt(0).toUpperCase() + k.slice(1)];
        if (val !== undefined && val !== null) return toNumber(val);
    }
    return 0;
}

// 1. Navigation
function showSection(sectionId) {
    console.log("Navigating to:", sectionId);
    
    // Hide all sections
    const sections = document.querySelectorAll('section');
    sections.forEach(s => s.classList.add('hidden'));
    
    // Show target section
    const target = document.getElementById(sectionId);
    if (target) {
        target.classList.remove('hidden');
    } else {
        console.warn("Section not found:", sectionId);
        // Default to dashboard if section not found
        const dash = document.getElementById('dashboard');
        if (dash) dash.classList.remove('hidden');
    }
    
    // Update active nav link
    document.querySelectorAll('.nav-links li').forEach(li => {
        li.classList.remove('active');
        const onclickAttr = li.getAttribute('onclick') || "";
        if (onclickAttr.includes(`'${sectionId}'`) || onclickAttr.includes(`"${sectionId}"`)) {
            li.classList.add('active');
        }
    });
    
    // Close sidebar on mobile
    const sidebar = document.getElementById('sidebar');
    const overlay = document.getElementById('sidebar-overlay');
    if (window.innerWidth <= 1024 && sidebar && overlay) {
        sidebar.classList.remove('open');
        overlay.classList.remove('active');
    }
}

function toggleSidebar() {
    const sidebar = document.getElementById('sidebar');
    const overlay = document.getElementById('sidebar-overlay');
    sidebar.classList.toggle('open');
    overlay.classList.toggle('active');
}

// 2. Food Search & Modal
let targetMealId = null;
let searchTimeout = null;
let modalSearchResults = [];
let recommendationResults = [];

async function openFoodModal(mealId) {
    targetMealId = mealId;
    document.getElementById('food-modal').classList.add('active');
    document.getElementById('food-modal-search').value = '';
    document.getElementById('modal-search-results').innerHTML = '';
    modalSearchResults = [];
    setTimeout(() => document.getElementById('food-modal-search').focus(), 100);
}

function closeFoodModal() { document.getElementById('food-modal').classList.remove('active'); }

function debounceModalSearch() {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(searchFoods, 300);
}

async function searchFoods() {
    const q = document.getElementById('food-modal-search').value;
    if (q.length < 2) return;
    
    try {
        modalSearchResults = await api.searchFoodsByName(q);
        const list = document.getElementById('modal-search-results');
        list.innerHTML = modalSearchResults.map((f, index) => {
            const energy = getMacroValue(f.nutrients, 'energy');
            return `
                <div class="card" style="margin-bottom:10px; cursor:pointer; padding:15px;" onclick="addFoodByIndex(${index})">
                    <div style="display:flex; justify-content:space-between; align-items:center;">
                        <div>
                            <strong>${f.name}</strong><br>
                            <small style="color:#666">${f.category || 'Umum'}</small>
                        </div>
                        <div style="text-align:right">
                            <span style="color:var(--primary); font-weight:700;">${energy.toFixed(0)}</span> <small>kcal</small>
                        </div>
                    </div>
                </div>
            `;
        }).join('');
    } catch (e) { console.error(e); }
}

function addFoodByIndex(index) {
    const food = modalSearchResults[index];
    if (!food) return;
    addFood(food);
}

function addFood(food) {
    currentSessionFoods.push({ 
        id: Date.now() + Math.random(), 
        name: food.name, 
        amount: food.servingSize || 100, 
        servingSize: food.servingSize || 100,
        servingUnit: food.servingUnit || 'g',
        mealTime: targetMealId, 
        nutrients: food.nutrients 
    });
    renderTable(); closeFoodModal();
}

function removeFood(id) {
    currentSessionFoods = currentSessionFoods.filter(f => f.id != id);
    renderTable();
}

function updateAmount(id, val) {
    const f = currentSessionFoods.find(x => x.id == id);
    if (f) { f.amount = toNumber(val); updateTotals(); renderTable(); }
}

// 3. Rekomendasi Logic
async function loadNutrientMetadata() {
    try { 
        nutrientMetadata = await api.getNutrientList(); 
        renderFilters();
    } catch (e) { console.error(e); }
}

async function addFilterRow() {
    if (nutrientMetadata.length === 0) await loadNutrientMetadata();
    const id = Date.now();
    filters.push({ id, nutrient: nutrientMetadata[0]?.name || 'energi', operator: '<', value: 100 });
    renderFilters();
}

function renderFilters() {
    const container = document.getElementById('filter-container');
    if (!container) return;
    
    container.innerHTML = filters.map(f => {
        const meta = nutrientMetadata.find(n => n.name === f.nutrient) || { unit: '' };
        return `
        <div style="display:flex; gap:10px; margin-bottom:15px; align-items:center;">
            <select onchange="updateFilter(${f.id}, 'nutrient', this.value)" style="flex:2;">
                ${nutrientMetadata.map(n => `<option value="${n.name}" ${f.nutrient === n.name ? 'selected' : ''}>${n.name}</option>`).join('')}
            </select>
            <select onchange="updateFilter(${f.id}, 'operator', this.value)" style="flex:1;">
                <option value="<" ${f.operator=='<'?'selected':''}>&lt;</option>
                <option value=">" ${f.operator=='>'?'selected':''}>&gt;</option>
                <option value="=" ${f.operator=='='?'selected':''}>=</option>
            </select>
            <input type="number" value="${f.value}" onchange="updateFilter(${f.id}, 'value', this.value)" style="flex:1;">
            <span style="font-size:0.8rem; min-width:30px;">${meta.unit}</span>
            <button class="btn-icon" onclick="removeFilter(${f.id})" style="color:red"><i class="fas fa-trash"></i></button>
        </div>`;
    }).join('');
}

function updateFilter(id, k, v) { 
    const f = filters.find(x => x.id === id); 
    if(f) { f[k] = (k === 'value') ? parseFloat(v) : v; if(k === 'nutrient') renderFilters(); }
}
function removeFilter(id) { filters = filters.filter(x => x.id !== id); renderFilters(); }

async function fetchRecommendations() {
    const resDiv = document.getElementById('rec-list');
    resDiv.innerHTML = '<div style="padding:20px; text-align:center;"><i class="fas fa-spinner fa-spin"></i> Mencari...</div>';
    try {
        recommendationResults = await api.getRecommendations(filters);
        resDiv.innerHTML = recommendationResults.map((f, index) => `
            <div class="card" style="margin-bottom:12px; display:flex; justify-content:space-between; align-items:center;">
                <div><strong>${f.name}</strong><br><small>${f.category || 'Umum'}</small></div>
                <button class="btn-add" onclick="addRecByIndexToSession(${index})">+ Tambah</button>
            </div>
        `).join('') || '<p style="text-align:center; color:#999;">Tidak ada hasil yang cocok.</p>';
    } catch (e) { resDiv.innerHTML = '<p style="color:red; text-align:center;">Gagal mengambil rekomendasi.</p>'; }
}

function addRecByIndexToSession(index) {
    const f = recommendationResults[index];
    if (!f) return;
    addRecToSession(f);
}

function addRecToSession(f) { 
    const food = {
        ...f,
        id: Date.now() + Math.random(),
        amount: f.servingSize || 100,
        servingSize: f.servingSize || 100,
        servingUnit: f.servingUnit || 'g',
        mealTime: activeMealTimes[0].id
    };
    currentSessionFoods.push(food); renderTable(); showSection('dashboard'); 
    showToast(`Ditambahkan ke ${activeMealTimes[0].label}`);
}

// 4. Persistence
function exportProject() {
    const data = { foods: currentSessionFoods, meals: activeMealTimes, targets: targets };
    const blob = new Blob([JSON.stringify(data)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    
    // Format date: yyyy-mm-dd (safe for filename)
    const now = new Date();
    const y = now.getFullYear();
    const m = String(now.getMonth() + 1).padStart(2, '0');
    const d = String(now.getDate()).padStart(2, '0');
    const dateStr = `${y}-${m}-${d}`;
    
    const a = document.createElement('a'); 
    a.href = url; 
    a.download = `Nutri-${dateStr}.nutri`; 
    a.click();
}

function importProject(event) {
    const reader = new FileReader();
    reader.onload = (e) => {
        try {
            const data = JSON.parse(e.target.result);
            currentSessionFoods = data.foods || []; activeMealTimes = data.meals || activeMealTimes;
            if (data.targets) targets = data.targets;
            renderTable(); showSection('dashboard'); showToast('Proyek berhasil diimpor!');
        } catch (err) { showToast('File tidak valid', 'error'); }
    };
    reader.readAsText(event.target.files[0]);
}

async function exportToWord() {
    try {
        const blob = await api.exportToWord({ foods: currentSessionFoods, mealTimes: activeMealTimes, targets: targets });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a'); a.href = url; a.download = 'Laporan_Nutrisi.rtf'; a.click();
    } catch (e) { showToast('Gagal ekspor Word', 'error'); }
}

// 5. Totals & Rendering
function updateTotals() {
    const totals = {};
    let sessionEnergy = 0, sessionCarbs = 0, sessionProtein = 0, sessionFat = 0;

    currentSessionFoods.forEach(f => {
        const baseSize = toNumber(f.servingSize) || 100;
        const r = toNumber(f.amount) / baseSize;
        sessionEnergy += getMacroValue(f.nutrients, 'energy') * r;
        sessionCarbs += getMacroValue(f.nutrients, 'carbs') * r;
        sessionProtein += getMacroValue(f.nutrients, 'protein') * r;
        sessionFat += getMacroValue(f.nutrients, 'fat') * r;

        Object.keys(f.nutrients).forEach(k => {
            const key = k.toLowerCase();
            totals[key] = (totals[key] || 0) + (toNumber(f.nutrients[k]) * r);
        });
    });

    document.getElementById('total-kcal').innerText = targets.kcal > 0
        ? `${sessionEnergy.toFixed(0)} / ${targets.kcal.toFixed(0)}`
        : sessionEnergy.toFixed(0);
    document.getElementById('total-carbs').innerText = targets.carbs > 0
        ? `${sessionCarbs.toFixed(1)}g / ${targets.carbs.toFixed(1)}g`
        : sessionCarbs.toFixed(1) + 'g';
    document.getElementById('total-protein').innerText = targets.protein > 0
        ? `${sessionProtein.toFixed(1)}g / ${targets.protein.toFixed(1)}g`
        : sessionProtein.toFixed(1) + 'g';
    document.getElementById('total-fat').innerText = targets.fat > 0
        ? `${sessionFat.toFixed(1)}g / ${targets.fat.toFixed(1)}g`
        : sessionFat.toFixed(1) + 'g';

    if (targets.kcal > 0) {
        document.getElementById('progress-kcal').style.width = Math.min((sessionEnergy / targets.kcal) * 100, 100) + '%';
        document.getElementById('progress-carbs').style.width = Math.min((sessionCarbs / targets.carbs) * 100, 100) + '%';
        document.getElementById('progress-protein').style.width = Math.min((sessionProtein / targets.protein) * 100, 100) + '%';
        document.getElementById('progress-fat').style.width = Math.min((sessionFat / targets.fat) * 100, 100) + '%';
    }

    const microSection = document.getElementById('micro-nutrient-section');
    const excludeKeys = Object.values(MACRO_MAP).flat().concat(['jumlah sajian', 'per sajian', 'kategori']);
    let microHtml = '';
    Object.keys(totals).sort().forEach(key => {
        if (excludeKeys.includes(key)) return;
        if (totals[key] <= 0) return;
        const meta = nutrientMetadata.find(n => n.name.toLowerCase() === key);
        microHtml += `<div class="summary-item"><span>${key.charAt(0).toUpperCase() + key.slice(1)}</span> <span>${totals[key].toFixed(2)} ${meta?meta.unit:''}</span></div>`;
    });
    microSection.innerHTML = microHtml || '<p style="color:#999; font-size:0.9rem;">Belum ada data mikronutrien.</p>';
}

function renderTable() {
    const table = document.getElementById('food-table');
    table.querySelectorAll('tbody').forEach(tb => tb.remove());

    activeMealTimes.forEach(meal => {
        const tbody = document.createElement('tbody');
        tbody.id = `meal-${meal.id}`;
        tbody.dataset.mealId = meal.id;
        tbody.className = 'meal-section';
        
        let html = `<tr class="meal-header-row"><td colspan="7"><div style="display:flex; justify-content:space-between; align-items:center;"><span><strong class="meal-drag-handle" style="cursor:move;"><i class="fas fa-grip-vertical" style="margin-right:10px; color:var(--primary);"></i>${meal.label}</strong></span><div style="display:flex; gap:8px;"><button class="btn-icon" onclick="openFoodModal('${meal.id}')"><i class="fas fa-plus"></i></button><button class="btn-icon" onclick="removeMealSection('${meal.id}')" style="color:red;"><i class="fas fa-trash"></i></button></div></div></td></tr>`;
        
        currentSessionFoods.filter(f => f.mealTime === meal.id).forEach(f => {
            const baseSize = toNumber(f.servingSize) || 100;
            const r = toNumber(f.amount) / baseSize;
            html += `
            <tr class="food-row" data-id="${f.id}">
                <td><i class="fas fa-grip-lines drag-handle" style="cursor:grab; margin-right:10px; color:#ccc;"></i>${f.name}</td>
                <td>
                    <div style="display:flex; align-items:center; gap:5px;">
                        <input type="number" value="${f.amount}" onchange="updateAmount(${f.id}, this.value)" style="width:60px">
                        <small>${f.servingUnit || 'g'}</small>
                    </div>
                </td>
                <td>${(getMacroValue(f.nutrients, 'energy') * r).toFixed(0)}</td>
                <td>${(getMacroValue(f.nutrients, 'carbs') * r).toFixed(1)}</td>
                <td>${(getMacroValue(f.nutrients, 'protein') * r).toFixed(1)}</td>
                <td>${(getMacroValue(f.nutrients, 'fat') * r).toFixed(1)}</td>
                <td><button class="btn-icon" onclick="removeFood(${f.id})" style="color:red"><i class="fas fa-trash"></i></button></td>
            </tr>`;
        });
        tbody.innerHTML = html;
        table.appendChild(tbody);

        if (window.Sortable) {
            new Sortable(tbody, {
                group: 'shared', animation: 150, handle: '.drag-handle', draggable: '.food-row',
                onEnd: function(evt) {
                    const itemEl = evt.item;
                    const newMealId = evt.to.dataset.mealId;
                    const foodId = parseFloat(itemEl.dataset.id);
                    const food = currentSessionFoods.find(x => x.id === foodId);
                    if (food) { food.mealTime = newMealId; updateTotals(); }
                }
            });
        }
    });

    // Reorder Meal Sections (tbody)
    if (window.Sortable) {
        if (table._sortableInstance) table._sortableInstance.destroy();
        table._sortableInstance = new Sortable(table, {
            animation: 150,
            handle: '.meal-drag-handle',
            draggable: 'tbody.meal-section',
            onEnd: function() {
                const newOrder = [];
                table.querySelectorAll('tbody.meal-section').forEach(tb => {
                    const mealId = tb.dataset.mealId;
                    const meal = activeMealTimes.find(m => m.id === mealId);
                    if (meal) newOrder.push(meal);
                });
                activeMealTimes = newOrder;
                updateTotals();
            }
        });
    }

    updateTotals();
}



// 6. TDEE & Integration
function validatePerc() {
    const c = parseFloat(document.getElementById('target-perc-carbs').value) || 0;
    const p = parseFloat(document.getElementById('target-perc-protein').value) || 0;
    const f = parseFloat(document.getElementById('target-perc-fat').value) || 0;
    const errorDiv = document.getElementById('perc-error');
    const valid = Math.abs((c + p + f) - 100) < 0.1;
    errorDiv.style.display = valid ? 'none' : 'block';
    return valid;
}

async function calculateAndApplyTargets() {
    if (!validatePerc()) return;
    const data = {
        gender: document.getElementById('tdee-gender').value,
        weightKg: parseFloat(document.getElementById('tdee-weight').value),
        heightCm: parseFloat(document.getElementById('tdee-height').value),
        age: parseInt(document.getElementById('tdee-age').value),
        activityFactor: parseFloat(document.getElementById('tdee-af-manual').value),
        injuryFactor: parseFloat(document.getElementById('tdee-if-manual').value)
    };

    try {
        const res = await api.calculateTdee(data);
        const pc = parseFloat(document.getElementById('target-perc-carbs').value) / 100;
        const pp = parseFloat(document.getElementById('target-perc-protein').value) / 100;
        const pf = parseFloat(document.getElementById('target-perc-fat').value) / 100;

        targets = { kcal: res.totalDailyEnergyExpenditure, carbs: (res.totalDailyEnergyExpenditure * pc) / 4, protein: (res.totalDailyEnergyExpenditure * pp) / 4, fat: (res.totalDailyEnergyExpenditure * pf) / 9 };
        
        document.getElementById('tdee-results').style.display = 'block';
        document.getElementById('res-tdee').innerText = targets.kcal.toFixed(0) + ' kcal';
        document.getElementById('res-carbs').innerText = targets.carbs.toFixed(1) + ' g';
        document.getElementById('res-protein').innerText = targets.protein.toFixed(1) + ' g';
        document.getElementById('res-fat').innerText = targets.fat.toFixed(1) + ' g';
        updateTotals(); showToast('Target nutrisi diperbarui!');
    } catch (e) { showToast('Gagal hitung TDEE', 'error'); }
}

async function importCsv() {
    const file = document.getElementById('csv-file').files[0];
    if (!file) return showToast('Pilih file CSV!', 'error');
    const formData = new FormData(); formData.append('file', file);
    try {
        await api.importCsv(formData); showToast('Database berhasil diimpor!');
    } catch (e) { showToast('Gagal impor database', 'error'); }
}

function openMealSectionModal() { document.getElementById('meal-modal').classList.add('active'); }
function createNewMealSection() {
    const name = document.getElementById('new-meal-name').value.trim();
    if(name) { 
        activeMealTimes.push({ id: generateMealId(), label: name }); 
        renderTable(); document.getElementById('meal-modal').classList.remove('active'); 
        document.getElementById('new-meal-name').value = '';
    }
}

function removeMealSection(mealId) {
    if (activeMealTimes.length <= 1) {
        showToast('Minimal harus ada 1 waktu makan', 'error');
        return;
    }

    const fallbackMeal = activeMealTimes.find(m => m.id !== mealId);
    if (!fallbackMeal) return;

    currentSessionFoods.forEach(f => {
        if (f.mealTime === mealId) f.mealTime = fallbackMeal.id;
    });

    activeMealTimes = activeMealTimes.filter(m => m.id !== mealId);
    renderTable();
    showToast(`Waktu makan dihapus. Item dipindah ke ${fallbackMeal.label}.`);
}

function showToast(msg, type = 'success') {
    const toast = document.createElement('div');
    toast.style = `position:fixed; bottom:20px; right:20px; padding:12px 24px; border-radius:12px; background:${type==='success'?'var(--primary)':'#e74c3c'}; color:white; z-index:3000; box-shadow:0 4px 12px rgba(0,0,0,0.1);`;
    toast.innerText = msg; document.body.appendChild(toast);
    setTimeout(() => toast.remove(), 3000);
}

window.onload = async () => { await loadNutrientMetadata(); renderTable(); addFilterRow(); };
