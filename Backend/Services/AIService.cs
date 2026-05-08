using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using NutritionApp.Models.AI;

namespace NutritionApp.Services
{
    public class AIService : IAIService
    {
        private readonly HttpClient _httpClient;
        private static readonly JsonSerializerOptions JsonOptions = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
        };

        public AIService(HttpClient httpClient)
        {
            _httpClient = httpClient;
        }

        public async Task<AIPromptResponse> GetMealPlanFromAIAsync(AIIntegrationRequest request)
        {
            if (request.AIConfig.Provider.Equals("Google", StringComparison.OrdinalIgnoreCase))
            {
                return await GetMealPlanFromGoogleAsync(request);
            }

            if (request.AIConfig.Provider.Equals("Anthropic", StringComparison.OrdinalIgnoreCase))
            {
                return await GetMealPlanFromAnthropicAsync(request);
            }

            return await GetMealPlanFromOpenAICompatibleAsync(request);
        }

        private async Task<AIPromptResponse> GetMealPlanFromOpenAICompatibleAsync(AIIntegrationRequest request)
        {
            var endpoint = BuildOpenAICompatibleEndpoint(request.AIConfig.BaseUrl);
            using var httpRequest = new HttpRequestMessage(HttpMethod.Post, endpoint);
            AddAuthorizationHeader(httpRequest, request.AIConfig.ApiKey);

            var payload = new
            {
                model = request.AIConfig.Model,
                messages = BuildMessages(request),
                temperature = 0.2
            };

            httpRequest.Content = CreateJsonContent(payload);
            using var response = await _httpClient.SendAsync(httpRequest);
            var responseBody = await response.Content.ReadAsStringAsync();
            Console.WriteLine("RAW AI RESPONSE: " + responseBody);
            response.EnsureSuccessStatusCode();

            using var json = JsonDocument.Parse(responseBody);
            if (!json.RootElement.TryGetProperty("choices", out var choices) || choices.GetArrayLength() == 0)
            {
                throw new InvalidOperationException("AI response does not contain choices.");
            }

            var message = choices[0].GetProperty("message");
            if (!message.TryGetProperty("content", out var contentElement))
            {
                throw new InvalidOperationException("AI response does not contain message content.");
            }

            return DeserializeMealPlan(contentElement.GetString());
        }

        private async Task<AIPromptResponse> GetMealPlanFromGoogleAsync(AIIntegrationRequest request)
        {
            var endpoint = BuildGoogleEndpoint(request.AIConfig.BaseUrl, request.AIConfig.Model, request.AIConfig.ApiKey);
            using var httpRequest = new HttpRequestMessage(HttpMethod.Post, endpoint);

            var payload = new
            {
                contents = new[]
                {
                    new
                    {
                        role = "user",
                        parts = new[] { new { text = BuildPrompt(request) } }
                    }
                },
                generationConfig = new { temperature = 0.2 }
            };

            httpRequest.Content = CreateJsonContent(payload);
            using var response = await _httpClient.SendAsync(httpRequest);
            var responseBody = await response.Content.ReadAsStringAsync();
            Console.WriteLine("RAW AI RESPONSE: " + responseBody);
            response.EnsureSuccessStatusCode();

            using var json = JsonDocument.Parse(responseBody);
            if (!json.RootElement.TryGetProperty("candidates", out var candidates) || candidates.GetArrayLength() == 0)
            {
                throw new InvalidOperationException("Google response does not contain candidates.");
            }

            var parts = candidates[0].GetProperty("content").GetProperty("parts");
            if (parts.GetArrayLength() == 0 || !parts[0].TryGetProperty("text", out var textElement))
            {
                throw new InvalidOperationException("Google response does not contain text content.");
            }

            return DeserializeMealPlan(textElement.GetString());
        }

        private async Task<AIPromptResponse> GetMealPlanFromAnthropicAsync(AIIntegrationRequest request)
        {
            var endpoint = BuildAnthropicEndpoint(request.AIConfig.BaseUrl);
            using var httpRequest = new HttpRequestMessage(HttpMethod.Post, endpoint);
            httpRequest.Headers.Add("x-api-key", request.AIConfig.ApiKey);
            httpRequest.Headers.Add("anthropic-version", "2023-06-01");

            var payload = new
            {
                model = request.AIConfig.Model,
                max_tokens = 4096,
                temperature = 0.2,
                system = "You are a nutrition meal-planning assistant. Always return valid JSON only.",
                messages = new[]
                {
                    new { role = "user", content = BuildPrompt(request) }
                }
            };

            httpRequest.Content = CreateJsonContent(payload);
            using var response = await _httpClient.SendAsync(httpRequest);
            var responseBody = await response.Content.ReadAsStringAsync();
            Console.WriteLine("RAW AI RESPONSE: " + responseBody);
            response.EnsureSuccessStatusCode();

            using var json = JsonDocument.Parse(responseBody);
            if (!json.RootElement.TryGetProperty("content", out var content) || content.GetArrayLength() == 0)
            {
                throw new InvalidOperationException("Anthropic response does not contain content.");
            }

            if (!content[0].TryGetProperty("text", out var textElement))
            {
                throw new InvalidOperationException("Anthropic response does not contain text content.");
            }

            return DeserializeMealPlan(textElement.GetString());
        }

        private static object[] BuildMessages(AIIntegrationRequest request)
        {
            var prompt = BuildPrompt(request);

            return new object[]
            {
                new { role = "system", content = "You are a nutrition meal-planning assistant. Always return valid JSON only." },
                new { role = "user", content = prompt }
            };
        }

        private static string BuildPrompt(AIIntegrationRequest request)
        {
            return
                $"Buat rencana makan harian untuk target {request.TargetTDEE} kkal.\n" +
                $"Target makro absolut: Karbohidrat {request.TargetCarbs}g, Protein {request.TargetProtein}g, Lemak {request.TargetFat}g.\n" +
                $"Waktu makan WAJIB menggunakan kategori berikut SAJA: {string.Join(", ", request.AvailableMealTypes)}.\n" +
                "Bagilah makanan ke dalam kategori-kategori tersebut secara logis. DILARANG menggunakan istilah waktu makan lain di luar daftar tersebut.\n" +
                "Kembalikan hanya JSON valid dengan bentuk:\n" +
                "{\"meal_plan\":[{\"meal_type\":\"Makan Pagi\",\"food_keyword\":\"Dada Ayam\",\"suggested_grams\":100,\"reasoning\":\"alasan singkat\"}]}\n" +
                "food_keyword harus berupa nama bahan/makanan dalam bahasa Indonesia untuk dicocokkan dengan database SQLite internal.\n" +
                "Jangan tambah markdown, komentar, atau teks di luar JSON.";
        }

        private static StringContent CreateJsonContent(object payload)
        {
            var json = JsonSerializer.Serialize(payload, JsonOptions);
            return new StringContent(json, Encoding.UTF8, "application/json");
        }

        private static void AddAuthorizationHeader(HttpRequestMessage request, string apiKey)
        {
            if (!string.IsNullOrWhiteSpace(apiKey))
            {
                request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", apiKey);
            }
        }

        private static string BuildOpenAICompatibleEndpoint(string baseUrl)
        {
            var normalized = NormalizeBaseUrl(baseUrl);
            if (normalized.EndsWith("/chat/completions", StringComparison.OrdinalIgnoreCase))
            {
                return normalized;
            }

            return normalized.TrimEnd('/') + "/chat/completions";
        }

        private static string BuildGoogleEndpoint(string baseUrl, string model, string apiKey)
        {
            var normalized = NormalizeBaseUrl(baseUrl);
            if (normalized.Contains(":generateContent", StringComparison.OrdinalIgnoreCase))
            {
                return normalized.Contains("?", StringComparison.Ordinal) ? normalized : normalized + "?key=" + Uri.EscapeDataString(apiKey);
            }

            return normalized.TrimEnd('/') + "/models/" + Uri.EscapeDataString(model) + ":generateContent?key=" + Uri.EscapeDataString(apiKey);
        }

        private static string BuildAnthropicEndpoint(string baseUrl)
        {
            var normalized = NormalizeBaseUrl(baseUrl);
            if (normalized.EndsWith("/messages", StringComparison.OrdinalIgnoreCase))
            {
                return normalized;
            }

            return normalized.TrimEnd('/') + "/messages";
        }

        private static string NormalizeBaseUrl(string baseUrl)
        {
            if (string.IsNullOrWhiteSpace(baseUrl))
            {
                throw new InvalidOperationException("AI baseUrl is required.");
            }

            return baseUrl.Trim();
        }

        private static AIPromptResponse DeserializeMealPlan(string? content)
        {
            if (string.IsNullOrWhiteSpace(content))
            {
                throw new InvalidOperationException("AI message content is empty.");
            }

            var parsed = JsonSerializer.Deserialize<AIPromptResponse>(content, JsonOptions);
            if (parsed == null)
            {
                throw new InvalidOperationException("AI message content could not be parsed into meal plan DTO.");
            }

            return parsed;
        }
    }
}
