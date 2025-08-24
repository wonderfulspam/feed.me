/**
 * Article Manager - Client-side article state management with localStorage persistence
 */
class ArticleManager {
    constructor() {
        this.storageKey = 'feedme-article-states';
        this.states = this.loadStates();
        this.showHidden = false;
        this.init();
    }

    /**
     * Load article states from localStorage
     */
    loadStates() {
        try {
            const stored = localStorage.getItem(this.storageKey);
            return stored ? JSON.parse(stored) : {};
        } catch (error) {
            console.warn('Failed to load article states from localStorage:', error);
            return {};
        }
    }

    /**
     * Save article states to localStorage
     */
    saveStates() {
        try {
            localStorage.setItem(this.storageKey, JSON.stringify(this.states));
        } catch (error) {
            console.warn('Failed to save article states to localStorage:', error);
        }
    }

    /**
     * Get article state by URL
     */
    getState(articleUrl) {
        return this.states[articleUrl] || {
            read: false,
            starred: false,
            hidden: false
        };
    }

    /**
     * Update article state
     */
    setState(articleUrl, newState) {
        if (!this.states[articleUrl]) {
            this.states[articleUrl] = { read: false, starred: false, hidden: false };
        }
        
        Object.assign(this.states[articleUrl], newState);
        this.saveStates();
        this.updateArticleDisplay(articleUrl);
    }

    /**
     * Mark article as read when main link is clicked
     */
    markAsRead(articleUrl) {
        this.setState(articleUrl, { read: true });
    }

    /**
     * Toggle starred state
     */
    toggleStar(articleUrl) {
        const currentState = this.getState(articleUrl);
        this.setState(articleUrl, { starred: !currentState.starred });
    }

    /**
     * Toggle hidden state
     */
    toggleHidden(articleUrl) {
        const currentState = this.getState(articleUrl);
        this.setState(articleUrl, { hidden: !currentState.hidden });
    }

    /**
     * Update article visual display based on state
     */
    updateArticleDisplay(articleUrl) {
        const article = document.querySelector(`a[href="${articleUrl}"]`);
        if (!article) return;

        const state = this.getState(articleUrl);
        
        // Update CSS classes
        article.classList.toggle('article-read', state.read);
        article.classList.toggle('article-starred', state.starred);
        
        // Handle hidden state with show/hide toggle consideration
        const shouldHide = state.hidden && !this.showHidden;
        article.classList.toggle('article-hidden', shouldHide);

        // Update control buttons
        const controls = article.querySelector('.article-controls');
        if (controls) {
            const starBtn = controls.querySelector('.star-btn');
            const hideBtn = controls.querySelector('.hide-btn');
            
            if (starBtn) {
                starBtn.textContent = '★';
                starBtn.setAttribute('aria-pressed', state.starred);
            }
            
            if (hideBtn) {
                hideBtn.textContent = '👁';
                hideBtn.setAttribute('aria-pressed', state.hidden);
            }
        }
    }

    /**
     * Toggle show/hide hidden articles
     */
    toggleShowHidden() {
        this.showHidden = !this.showHidden;
        
        // Update toggle button
        const toggleBtn = document.getElementById('toggle-hidden');
        if (toggleBtn) {
            toggleBtn.textContent = this.showHidden ? 'Hide Hidden' : 'Show Hidden';
            toggleBtn.setAttribute('aria-pressed', this.showHidden);
        }
        
        // Update all article displays
        Object.keys(this.states).forEach(url => {
            this.updateArticleDisplay(url);
        });
    }

    /**
     * Create control buttons for an article
     */
    createControls(articleUrl) {
        const controls = document.createElement('div');
        controls.className = 'article-controls';
        controls.setAttribute('role', 'group');
        controls.setAttribute('aria-label', 'Article actions');

        const state = this.getState(articleUrl);

        // Star button
        const starBtn = document.createElement('button');
        starBtn.className = 'control-btn star-btn';
        starBtn.textContent = '★';
        starBtn.setAttribute('aria-label', state.starred ? 'Remove from favorites' : 'Add to favorites');
        starBtn.setAttribute('aria-pressed', state.starred);
        starBtn.addEventListener('click', (e) => {
            e.preventDefault();
            e.stopPropagation();
            this.toggleStar(articleUrl);
        });

        // Hide button
        const hideBtn = document.createElement('button');
        hideBtn.className = 'control-btn hide-btn';
        hideBtn.textContent = '👁';
        hideBtn.setAttribute('aria-label', state.hidden ? 'Show article' : 'Hide article');
        hideBtn.setAttribute('aria-pressed', state.hidden);
        hideBtn.addEventListener('click', (e) => {
            e.preventDefault();
            e.stopPropagation();
            this.toggleHidden(articleUrl);
        });

        controls.appendChild(starBtn);
        controls.appendChild(hideBtn);

        return controls;
    }

    /**
     * Initialize article manager
     */
    init() {
        // Wait for DOM to be ready
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => this.setupArticles());
        } else {
            this.setupArticles();
        }
    }

    /**
     * Setup all articles on the page
     */
    setupArticles() {
        // Setup toggle button
        const toggleBtn = document.getElementById('toggle-hidden');
        if (toggleBtn) {
            toggleBtn.addEventListener('click', () => {
                this.toggleShowHidden();
            });
        }
        
        // Find all article links and add controls
        const articleLinks = document.querySelectorAll('a[role="button"]');
        
        articleLinks.forEach(link => {
            const articleUrl = link.getAttribute('href');
            if (!articleUrl || articleUrl.startsWith('#')) return;

            // Add click handler to mark as read
            link.addEventListener('click', () => {
                this.markAsRead(articleUrl);
            });

            // Create and append control buttons
            const article = link.querySelector('article');
            if (article && !article.querySelector('.article-controls')) {
                const controls = this.createControls(articleUrl);
                article.appendChild(controls);
            }

            // Apply initial state
            this.updateArticleDisplay(articleUrl);
        });
    }

    /**
     * Get statistics about article states
     */
    getStats() {
        const stats = {
            total: Object.keys(this.states).length,
            read: 0,
            starred: 0,
            hidden: 0
        };

        Object.values(this.states).forEach(state => {
            if (state.read) stats.read++;
            if (state.starred) stats.starred++;
            if (state.hidden) stats.hidden++;
        });

        return stats;
    }

    /**
     * Export article states as JSON
     */
    exportStates() {
        return JSON.stringify(this.states, null, 2);
    }

    /**
     * Import article states from JSON
     */
    importStates(jsonString) {
        try {
            const importedStates = JSON.parse(jsonString);
            this.states = { ...this.states, ...importedStates };
            this.saveStates();
            
            // Refresh display for all articles
            this.setupArticles();
            
            return true;
        } catch (error) {
            console.error('Failed to import article states:', error);
            return false;
        }
    }
}

// Initialize article manager when script loads
const articleManager = new ArticleManager();

// Export to global scope for console access
window.articleManager = articleManager;