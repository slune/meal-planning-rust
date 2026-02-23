use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="space-y-12">
            <div class="text-center py-8">
                <div class="text-7xl mb-6">"🏕️"</div>
                <h1 class="text-5xl font-bold mb-6 text-blue-600">
                    "Boy Scout Meal Planner"
                </h1>
                <p class="text-2xl text-slate-600">"Organize your camp meals with ease and efficiency"</p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                <a href="/camps" class="card group">
                    <div class="text-5xl mb-4">"🏕️"</div>
                    <h3 class="text-2xl font-bold mb-3 text-blue-600">"Camps"</h3>
                    <p class="text-slate-700 text-lg leading-relaxed">"Manage your scout camps and events with dates and attendance tracking"</p>
                </a>

                <a href="/recipes" class="card group">
                    <div class="text-5xl mb-4">"🍳"</div>
                    <h3 class="text-2xl font-bold mb-3 text-blue-600">"Recipes"</h3>
                    <p class="text-slate-700 text-lg leading-relaxed">"Create and manage delicious recipes with portion calculations"</p>
                </a>

                <a href="/ingredients" class="card group">
                    <div class="text-5xl mb-4">"🥕"</div>
                    <h3 class="text-2xl font-bold mb-3 text-blue-600">"Ingredients"</h3>
                    <p class="text-slate-700 text-lg leading-relaxed">"Organize ingredients and categories for efficient planning"</p>
                </a>

                <a href="/planner" class="card group">
                    <div class="text-5xl mb-4">"📅"</div>
                    <h3 class="text-2xl font-bold mb-3 text-blue-600">"Meal Planner"</h3>
                    <p class="text-slate-700 text-lg leading-relaxed">"Plan daily meals for your camps with smart scheduling"</p>
                </a>

                <a href="/reports" class="card group">
                    <div class="text-5xl mb-4">"📊"</div>
                    <h3 class="text-2xl font-bold mb-3 text-blue-600">"Reports"</h3>
                    <p class="text-slate-700 text-lg leading-relaxed">"Generate shopping lists and comprehensive meal reports"</p>
                </a>

                <div class="card bg-blue-50 border-2 border-blue-200">
                    <div class="text-5xl mb-4">"✨"</div>
                    <h3 class="text-2xl font-bold mb-3 text-green-600">"Quick Tips"</h3>
                    <p class="text-slate-700 text-lg leading-relaxed">"Start with ingredients, build recipes, create camps, and plan meals!"</p>
                </div>
            </div>

            <div class="card bg-blue-50 border-2 border-blue-300">
                <div class="flex items-start gap-6">
                    <div class="text-6xl">"🚀"</div>
                    <div class="flex-1">
                        <h3 class="text-3xl font-bold mb-6 text-blue-600">"Getting Started Guide"</h3>
                        <ol class="space-y-5 text-slate-700">
                            <li class="flex items-start gap-4">
                                <span class="inline-flex items-center justify-center w-10 h-10 rounded-full bg-blue-600 text-white font-bold flex-shrink-0 text-lg">"1"</span>
                                <span class="pt-2 text-lg leading-relaxed">"Create ingredient categories and add ingredients to your inventory"</span>
                            </li>
                            <li class="flex items-start gap-4">
                                <span class="inline-flex items-center justify-center w-10 h-10 rounded-full bg-blue-600 text-white font-bold flex-shrink-0 text-lg">"2"</span>
                                <span class="pt-2 text-lg leading-relaxed">"Build your recipe library with accurate portion sizes and instructions"</span>
                            </li>
                            <li class="flex items-start gap-4">
                                <span class="inline-flex items-center justify-center w-10 h-10 rounded-full bg-blue-600 text-white font-bold flex-shrink-0 text-lg">"3"</span>
                                <span class="pt-2 text-lg leading-relaxed">"Set up a camp with dates, location, and default attendance numbers"</span>
                            </li>
                            <li class="flex items-start gap-4">
                                <span class="inline-flex items-center justify-center w-10 h-10 rounded-full bg-blue-600 text-white font-bold flex-shrink-0 text-lg">"4"</span>
                                <span class="pt-2 text-lg leading-relaxed">"Plan meals for each day using your recipe library"</span>
                            </li>
                            <li class="flex items-start gap-4">
                                <span class="inline-flex items-center justify-center w-10 h-10 rounded-full bg-blue-600 text-white font-bold flex-shrink-0 text-lg">"5"</span>
                                <span class="pt-2 text-lg leading-relaxed">"Generate shopping lists and daily reports for your camp"</span>
                            </li>
                        </ol>
                    </div>
                </div>
            </div>
        </div>
    }
}
