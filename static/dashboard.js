
let users = [];
let videos = [];
let interactionData = [];
let recommendationHistory = [];

document.addEventListener('DOMContentLoaded', function() {
    fetchSystemStats();
    initializeCharts();
    populateUserDropdown();
    
    document.getElementById('get-recommendations').addEventListener('click', fetchRecommendations);
    document.getElementById('run-simulation').addEventListener('click', runSimulation);
    
    setInterval(updateCharts, 60000);
});

async function fetchSystemStats() {
    try {
        const response = await fetch('/api/stats');
        const data = await response.json();
        
        document.getElementById('total-videos').textContent = data.videoCount;
        document.getElementById('total-users').textContent = data.userCount;
        document.getElementById('interactions-today').textContent = data.interactionsToday;
        document.getElementById('recommendation-quality').textContent = `${data.recommendationQuality.toFixed(1)}%`;
        
        users = data.users;
        videos = data.videos;
        interactionData = data.interactions;
        recommendationHistory = data.recommendationHistory;
        
    } catch (error) {
        console.error('Error fetching system stats:', error);
    }
}

function initializeCharts() {
    const interactionCtx = document.getElementById('interaction-chart').getContext('2d');
    const interactionChart = new Chart(interactionCtx, {
        type: 'bar',
        data: {
            labels: ['Likes', 'Dislikes', 'Comments', 'Shares', 'Subscriptions', 'Other'],
            datasets: [{
                label: 'Count',
                data: [0, 0, 0, 0, 0, 0],
                backgroundColor: [
                    'rgba(75, 192, 192, 0.6)',
                    'rgba(255, 99, 132, 0.6)',
                    'rgba(255, 206, 86, 0.6)',
                    'rgba(54, 162, 235, 0.6)',
                    'rgba(153, 102, 255, 0.6)',
                    'rgba(255, 159, 64, 0.6)'
                ]
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false
        }
    });
    
    const watchTimeCtx = document.getElementById('watch-time-chart').getContext('2d');
    const watchTimeChart = new Chart(watchTimeCtx, {
        type: 'line',
        data: {
            labels: ['0-25%', '25-50%', '50-75%', '75-100%'],
            datasets: [{
                label: 'Distribution',
                data: [0, 0, 0, 0],
                borderColor: 'rgba(75, 192, 192, 1)',
                backgroundColor: 'rgba(75, 192, 192, 0.2)',
                tension: 0.4,
                fill: true
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false
        }
    });
    
    const engagementCtx = document.getElementById('engagement-chart').getContext('2d');
    const engagementChart = new Chart(engagementCtx, {
        type: 'line',
        data: {
            labels: Array.from({length: 7}, (_, i) => {
                const d = new Date();
                d.setDate(d.getDate() - (6 - i));
                return d.toLocaleDateString();
            }),
            datasets: [{
                label: 'Likes',
                data: [0, 0, 0, 0, 0, 0, 0],
                borderColor: 'rgba(75, 192, 192, 1)',
                backgroundColor: 'transparent'
            }, {
                label: 'Comments',
                data: [0, 0, 0, 0, 0, 0, 0],
                borderColor: 'rgba(255, 206, 86, 1)',
                backgroundColor: 'transparent'
            }, {
                label: 'Shares',
                data: [0, 0, 0, 0, 0, 0, 0],
                borderColor: 'rgba(54, 162, 235, 1)',
                backgroundColor: 'transparent'
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false
        }
    });
    
    const categoriesCtx = document.getElementById('categories-chart').getContext('2d');
    const categoriesChart = new Chart(categoriesCtx, {
        type: 'doughnut',
        data: {
            labels: ['Category 1', 'Category 2', 'Category 3', 'Category 4', 'Others'],
            datasets: [{
                data: [0, 0, 0, 0, 0],
                backgroundColor: [
                    'rgba(75, 192, 192, 0.6)',
                    'rgba(255, 99, 132, 0.6)',
                    'rgba(255, 206, 86, 0.6)',
                    'rgba(54, 162, 235, 0.6)',
                    'rgba(153, 102, 255, 0.6)'
                ]
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false
        }
    });
    
    window.dashboardCharts = {
        interactionChart,
        watchTimeChart,
        engagementChart,
        categoriesChart
    };
}

async function updateCharts() {
    try {
        const response = await fetch('/api/chart-data');
        const data = await response.json();
        
        window.dashboardCharts.interactionChart.data.datasets[0].data = [
            data.interactions.likes,
            data.interactions.dislikes,
            data.interactions.comments,
            data.interactions.shares,
            data.interactions.subscriptions,
            data.interactions.other
        ];
        window.dashboardCharts.interactionChart.update();
        
        window.dashboardCharts.watchTimeChart.data.datasets[0].data = data.watchTimeDistribution;
        window.dashboardCharts.watchTimeChart.update();
        
        for (let i = 0; i < 3; i++) {
            window.dashboardCharts.engagementChart.data.datasets[i].data = data.engagementTimeline[i];
        }
        window.dashboardCharts.engagementChart.update();
        
        window.dashboardCharts.categoriesChart.data.labels = data.categories.labels;
        window.dashboardCharts.categoriesChart.data.datasets[0].data = data.categories.values;
        window.dashboardCharts.categoriesChart.update();
        
    } catch (error) {
        console.error('Error updating charts:', error);
    }
}

function populateUserDropdown() {
    const userSelect = document.getElementById('user-select');
    
    const demoUsers = [
        { id: 'user1', name: 'Test User 1' },
        { id: 'user2', name: 'Test User 2' },
        { id: 'user3', name: 'Test User 3' }
    ];
    
    demoUsers.forEach(user => {
        const option = document.createElement('option');
        option.value = user.id;
        option.textContent = user.name;
        userSelect.appendChild(option);
    });
}

async function fetchRecommendations() {
    const userId = document.getElementById('user-select').value;
    if (!userId) {
        alert('Please select a user');
        return;
    }
    
    try {
        const response = await fetch('/api/recommendations', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ user_id: userId, count: 10 })
        });
        
        const recommendations = await response.json();
        displayRecommendations(recommendations);
        
    } catch (error) {
        console.error('Error fetching recommendations:', error);
        alert('Failed to fetch recommendations');
    }
}

function displayRecommendations(recommendations) {
    const container = document.getElementById('recommendation-container');
    container.innerHTML = '';
    
    if (!recommendations || recommendations.length === 0) {
        container.innerHTML = '<div class="col-12"><p>No recommendations found.</p></div>';
        return;
    }
    
    recommendations.forEach(video => {
        const card = document.createElement('div');
        card.className = 'col-md-6 col-lg-4';
        
        card.innerHTML = `
            <div class="card h-100">
                <div class="card-body">
                    <h5 class="card-title">${video.title}</h5>
                    <p class="card-text">
                        <small>
                            <span class="badge bg-primary">${video.categories.join(', ')}</span>
                        </small>
                    </p>
                    <div class="d-flex justify-content-between">
                        <span><i class="bi bi-eye"></i> ${video.metrics.views}</span>
                        <span><i class="bi bi-hand-thumbs-up"></i> ${video.metrics.likes}</span>
                        <span><i class="bi bi-chat"></i> ${video.metrics.comment_count}</span>
                    </div>
                </div>
                <div class="card-footer">
                    <button class="btn btn-sm btn-outline-primary simulate-view" data-video-id="${video.id}">
                        Simulate View
                    </button>
                    <button class="btn btn-sm btn-outline-success simulate-like" data-video-id="${video.id}">
                        Like
                    </button>
                </div>
            </div>
        `;
        
        container.appendChild(card);
    });
    
    document.querySelectorAll('.simulate-view').forEach(button => {
        button.addEventListener('click', () => {
            simulateView(button.getAttribute('data-video-id'));
        });
    });
    
    document.querySelectorAll('.simulate-like').forEach(button => {
        button.addEventListener('click', () => {
            simulateLike(button.getAttribute('data-video-id'));
        });
    });
}

async function simulateView(videoId) {
    const userId = document.getElementById('user-select').value;
    try {
        await fetch('/api/watch', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                user_id: userId,
                video_id: videoId,
                watch_seconds: Math.floor(Math.random() * 300) + 60
            })
        });
        
        alert('View simulated successfully!');
    } catch (error) {
        console.error('Error simulating view:', error);
    }
}

async function simulateLike(videoId) {
    const userId = document.getElementById('user-select').value;
    try {
        await fetch('/api/like', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                user_id: userId,
                video_id: videoId,
                is_like: true
            })
        });
        
        alert('Like simulated successfully!');
    } catch (error) {
        console.error('Error simulating like:', error);
    }
}

async function runSimulation() {
    const userCount = parseInt(document.getElementById('simulation-users').value) || 100;
    const days = parseInt(document.getElementById('simulation-days').value) || 7;
    const intensity = document.getElementById('simulation-intensity').value;
    
    const progressBar = document.querySelector('#simulation-progress .progress-bar');
    const simulationProgress = document.getElementById('simulation-progress');
    const resultsContainer = document.getElementById('simulation-results');
    
    simulationProgress.classList.remove('d-none');
    progressBar.style.width = '0%';
    resultsContainer.innerHTML = '<div class="alert alert-info">Simulation in progress...</div>';
    
    try {
        const response = await fetch('/api/simulate', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ user_count: userCount, days, intensity })
        });
        
        const simulationInterval = setInterval(() => {
            fetch('/api/simulation-status')
                .then(res => res.json())
                .then(status => {
                    const progress = status.progress;
                    progressBar.style.width = `${progress}%`;
                    
                    if (progress >= 100) {
                        clearInterval(simulationInterval);
                        completeSimulation();
                    }
                })
                .catch(err => {
                    console.error('Error checking simulation status:', err);
                });
        }, 1000);
        
    } catch (error) {
        console.error('Error starting simulation:', error);
        simulationProgress.classList.add('d-none');
        resultsContainer.innerHTML = '<div class="alert alert-danger">Simulation failed to start</div>';
    }
}

async function completeSimulation() {
    const resultsContainer = document.getElementById('simulation-results');
    
    try {
        const response = await fetch('/api/simulation-results');
        const results = await response.json();
        
        resultsContainer.innerHTML = `
            <div class="alert alert-success">
                <h5>Simulation Completed</h5>
                <p>Generated ${results.totalInteractions} interactions across ${results.userCount} users.</p>
            </div>
            <div class="row">
                <div class="col-md-6">
                    <h6>Interaction Totals:</h6>
                    <ul>
                        <li>Views: ${results.metrics.views}</li>
                        <li>Likes: ${results.metrics.likes}</li>
                        <li>Comments: ${results.metrics.comments}</li>
                        <li>Shares: ${results.metrics.shares}</li>
                        <li>Subscribes: ${results.metrics.subscribes}</li>
                    </ul>
                </div>
                <div class="col-md-6">
                    <h6>Recommendation Metrics:</h6>
                    <ul>
                        <li>CTR: ${(results.metrics.ctr * 100).toFixed(2)}%</li>
                        <li>Average Watch Time: ${results.metrics.avgWatchTime}s</li>
                        <li>Engagement Rate: ${(results.metrics.engagementRate * 100).toFixed(2)}%</li>
                    </ul>
                </div>
            </div>
        `;
        
        updateCharts();
        fetchSystemStats();
        
    } catch (error) {
        console.error('Error fetching simulation results:', error);
        resultsContainer.innerHTML = '<div class="alert alert-danger">Failed to load simulation results</div>';
    }
}