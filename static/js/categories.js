// Categories page functionality
document.addEventListener('DOMContentLoaded', function() {
    // Get current URL parameters
    const urlParams = new URLSearchParams(window.location.search);
    const selectedTag = urlParams.get('tag');
    
    if (selectedTag) {
        // Show filtered view for the selected tag
        showFilteredView(selectedTag);
    }
});

function showFilteredView(tagName) {
    // Hide the default all-categories view
    const allCategoriesSection = document.querySelector('.all-categories');
    if (allCategoriesSection) {
        allCategoriesSection.style.display = 'none';
    }
    
    // Create filtered results section
    const mainSection = document.querySelector('.categories-page');
    if (!mainSection) return;
    
    // Update page title and meta
    document.title = `${tagName} - Categories - Feed.me`;
    
    // Create filtered results HTML
    const filteredSection = document.createElement('div');
    filteredSection.className = 'filtered-results';
    filteredSection.innerHTML = `
        <h2>Articles tagged with "${tagName}"</h2>
        <div class="clear-filter">
            <a href="/categories">← View all categories</a>
        </div>
        <div id="filtered-articles" class="articles-grid">
            <p>Loading articles...</p>
        </div>
    `;
    
    mainSection.appendChild(filteredSection);
    
    // Highlight the active tag in the tag cloud
    const tagLinks = document.querySelectorAll('.tag-cloud .tag');
    tagLinks.forEach(tag => {
        const tagHref = tag.getAttribute('href');
        if (tagHref && tagHref.includes(`tag=${tagName}`)) {
            tag.classList.add('tag-active');
            tag.style.backgroundColor = '#0066cc';
            tag.style.color = 'white';
        }
    });
    
    // Load and display filtered articles
    loadFilteredArticles(tagName);
}

async function loadFilteredArticles(tagName) {
    try {
        // Load search data
        const response = await fetch('/data/searchData.json');
        const articles = await response.json();
        
        // Filter articles by tag
        const filteredArticles = articles.filter(article => 
            article.tags && article.tags.includes(tagName)
        );
        
        // Display filtered articles
        displayFilteredArticles(filteredArticles);
        
    } catch (error) {
        console.error('Error loading articles:', error);
        const container = document.getElementById('filtered-articles');
        if (container) {
            container.innerHTML = '<p>Error loading articles. Please try again.</p>';
        }
    }
}

function displayFilteredArticles(articles) {
    const container = document.getElementById('filtered-articles');
    if (!container) return;
    
    if (articles.length === 0) {
        container.innerHTML = '<p>No articles found for this tag.</p>';
        return;
    }
    
    // Sort articles by date (most recent first)
    articles.sort((a, b) => new Date(b.pub_date) - new Date(a.pub_date));
    
    // Generate HTML for articles
    const articlesHTML = articles.map(article => {
        const date = new Date(article.pub_date).toLocaleDateString();
        const tagsHTML = article.tags ? article.tags.map(tag => 
            `<a href="/categories?tag=${tag}" class="tag tag-${tag}">${tag}</a>`
        ).join('') : '';
        
        return `
            <a href="${article.item_url}" class="article-link" aria-label="Read article: ${article.title}">
                <article class="article" role="article">
                    <span class="author">${article.author}</span>
                    |
                    <span class="date">${date}</span>
                    <h3>${article.title}</h3>
                    <p>${article.safe_description}</p>
                    ${tagsHTML ? `<div class="tags">${tagsHTML}</div>` : ''}
                </article>
            </a>
        `;
    }).join('');
    
    container.innerHTML = articlesHTML;
}