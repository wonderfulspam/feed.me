class ArticleSearch {
  constructor() {
    this.articles = [];
    this.authors = new Set();
    this.searchInput = document.getElementById('search-input');
    this.authorFilter = document.getElementById('author-filter');
    this.tierFilter = document.getElementById('tier-filter');
    this.searchStatus = document.getElementById('search-status');
    this.searchResultsList = document.getElementById('search-results-list');
    
    this.loadSearchData();
    this.setupEventListeners();
  }
  
  async loadSearchData() {
    try {
      const response = await fetch('/data/searchData.json');
      if (!response.ok) {
        throw new Error('Failed to load search data');
      }
      
      this.articles = await response.json();
      this.populateAuthorFilter();
      this.updateSearchStatus('Search data loaded. Enter a search term to begin.');
      
      // Perform search if there's a query in the URL
      const urlParams = new URLSearchParams(window.location.search);
      const query = urlParams.get('q');
      if (query) {
        this.searchInput.value = query;
        this.performSearch();
      }
    } catch (error) {
      console.error('Error loading search data:', error);
      this.updateSearchStatus('Error loading search data. Please try again later.');
    }
  }
  
  populateAuthorFilter() {
    // Collect unique authors
    this.articles.forEach(article => {
      this.authors.add(article.author);
    });
    
    // Populate author filter dropdown
    const sortedAuthors = Array.from(this.authors).sort();
    sortedAuthors.forEach(author => {
      const option = document.createElement('option');
      option.value = author;
      option.textContent = author;
      this.authorFilter.appendChild(option);
    });
  }
  
  setupEventListeners() {
    // Debounced search on input
    let searchTimeout;
    this.searchInput.addEventListener('input', () => {
      clearTimeout(searchTimeout);
      searchTimeout = setTimeout(() => {
        this.performSearch();
      }, 300);
    });
    
    // Filter changes
    this.authorFilter.addEventListener('change', () => {
      this.performSearch();
    });
    
    this.tierFilter.addEventListener('change', () => {
      this.performSearch();
    });
    
    // Handle form submission (Enter key)
    this.searchInput.addEventListener('keypress', (e) => {
      if (e.key === 'Enter') {
        this.performSearch();
      }
    });
  }
  
  performSearch() {
    const query = this.searchInput.value.trim().toLowerCase();
    const authorFilter = this.authorFilter.value;
    const tierFilter = this.tierFilter.value;
    
    if (query === '' && authorFilter === '' && tierFilter === '') {
      this.updateSearchStatus('Enter a search term or select filters to search.');
      this.searchResultsList.innerHTML = '';
      return;
    }
    
    let filteredArticles = this.articles;
    
    // Apply filters
    if (authorFilter) {
      filteredArticles = filteredArticles.filter(article => 
        article.author === authorFilter
      );
    }
    
    if (tierFilter) {
      filteredArticles = filteredArticles.filter(article => 
        article.tier === tierFilter
      );
    }
    
    // Apply text search
    if (query) {
      filteredArticles = filteredArticles.filter(article => {
        const searchableText = [
          article.title,
          article.description,
          article.author
        ].join(' ').toLowerCase();
        
        return searchableText.includes(query);
      });
      
      // Simple relevance scoring (title matches score higher)
      filteredArticles = filteredArticles.map(article => {
        let score = 0;
        const titleLower = article.title.toLowerCase();
        const descriptionLower = article.description.toLowerCase();
        
        if (titleLower.includes(query)) {
          score += titleLower.split(query).length - 1; // Count occurrences
        }
        if (descriptionLower.includes(query)) {
          score += (descriptionLower.split(query).length - 1) * 0.5; // Lower weight for description
        }
        
        return { ...article, score };
      });
      
      // Sort by relevance score (descending)
      filteredArticles.sort((a, b) => b.score - a.score);
    } else {
      // Sort by date if no text search
      filteredArticles.sort((a, b) => 
        new Date(b.pub_date) - new Date(a.pub_date)
      );
    }
    
    this.displayResults(filteredArticles, query);
  }
  
  displayResults(results, query) {
    if (results.length === 0) {
      this.updateSearchStatus('No articles found matching your search criteria.');
      this.searchResultsList.innerHTML = '';
      return;
    }
    
    const plural = results.length === 1 ? '' : 's';
    this.updateSearchStatus(`Found ${results.length} article${plural}${query ? ` for "${query}"` : ''}.`);
    
    // Display results
    const resultsHtml = results.map(article => {
      const date = new Date(article.pub_date).toLocaleDateString();
      const description = article.description.length > 200 
        ? article.description.substring(0, 200) + '...' 
        : article.description;
      
      return `
        <article class="article-item">
          <header class="article-header">
            <h3 class="article-title">
              <a href="${article.item_url}" target="_blank" rel="noopener noreferrer">
                ${this.escapeHtml(article.title)}
              </a>
            </h3>
            <div class="article-meta">
              <span class="article-author">${this.escapeHtml(article.author)}</span>
              <span class="article-tier tier-${article.tier}">${article.tier}</span>
              <span class="article-date">${date}</span>
            </div>
          </header>
          <div class="article-description">
            ${this.escapeHtml(description)}
          </div>
        </article>
      `;
    }).join('');
    
    this.searchResultsList.innerHTML = resultsHtml;
  }
  
  updateSearchStatus(message) {
    this.searchStatus.textContent = message;
  }
  
  escapeHtml(text) {
    const map = {
      '&': '&amp;',
      '<': '&lt;',
      '>': '&gt;',
      '"': '&quot;',
      "'": '&#039;'
    };
    return text.replace(/[&<>"']/g, m => map[m]);
  }
}

// Initialize search when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  new ArticleSearch();
});