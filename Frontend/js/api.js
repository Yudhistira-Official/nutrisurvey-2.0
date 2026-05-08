const API_BASE = 'http://localhost:5000/api';

const api = {
    async searchFoodsByName(query) {
        const response = await fetch(`${API_BASE}/FoodSearch/search?query=${encodeURIComponent(query || "")}`);
        if (!response.ok) throw new Error('Search failed');
        return await response.json();
    },

    async getNutrientList() {
        const response = await fetch(`${API_BASE}/Nutrient/list`);
        if (!response.ok) throw new Error('Failed to fetch nutrients');
        return await response.json();
    },

    async getRecommendations(filters) {
        const response = await fetch(`${API_BASE}/FoodSearch/recommendations`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(filters)
        });
        if (!response.ok) throw new Error('Recommendation failed');
        return await response.json();
    },

    async calculateTdee(data) {
        const response = await fetch(`${API_BASE}/Nutrition/calculate-tdee`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });
        if (!response.ok) throw new Error('Calculation failed');
        return await response.json();
    },

    async generateAiMenu(data) {
        const response = await fetch(`${API_BASE}/AI/generate-menu`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });

        if (!response.ok) {
            let message = 'AI menu generation failed';
            try {
                const error = await response.json();
                message = error.message || message;
            } catch (_) { }
            throw new Error(message);
        }

        return await response.json();
    },

    async exportToWord(projectData) {
        const response = await fetch(`${API_BASE}/Export/word`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(projectData)
        });
        if (!response.ok) throw new Error('Export failed');
        return await response.blob();
    },

    async importCsv(formData) {
        const response = await fetch(`${API_BASE}/Import/food-db`, {
            method: 'POST',
            body: formData
        });
        if (!response.ok) throw new Error('Import failed');
        return await response.json();
    }
};
