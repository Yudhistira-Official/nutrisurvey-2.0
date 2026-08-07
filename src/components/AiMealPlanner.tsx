'use client';

import { useEffect, useMemo, useRef, useState } from 'react';
import { cancelAiRequest, generateAiMenu, getAiDefaultInfo, loadAiKey, streamAiMenu } from '../lib/commands';
import type { AiConfig, AiMealRow, MealTime, SessionFood, Targets, TdeeClinicalContext } from '../lib/types';

type ChatEntry = { role: 'user' | 'assistant'; content: string; streaming?: boolean; isError?: boolean; retryPrompt?: string; id: number };
type AssessmentAnswer = { id: string; value: string };
type PlannerSession = { rows: AiMealRow[]; chat: ChatEntry[]; assessmentAnswers: AssessmentAnswer[]; assessmentVisible: boolean; planStarted: boolean; clinicalContext: TdeeClinicalContext | null };
const storageKey = 'nutrisurvey.ai-planner-session';
const planRequest = 'Buat plan menu sekarang berdasarkan seluruh hasil asesmen saya. Susun menu harian sesuai target TDEE dan makro.';
const maxContextTokens = 200000;
const estimateTokens = (value: string) => Math.ceil(value.length / 4);
const providerNames: Record<string, string> = { builtin_default: 'AI Default', openrouter: 'OpenRouter', openai: 'OpenAI', google: 'Google', anthropic: 'Anthropic', custom: 'Custom Router' };
const assessmentSteps = [
  { id: 'FH', label: 'Riwayat Gizi', title: 'Bagaimana pola makan Anda?', hint: 'Tuliskan makanan/minuman yang biasa dikonsumsi, jadwal makan, alergi, pantangan, serta makanan yang disukai atau tidak disukai.' },
  { id: 'CH', label: 'Riwayat Klien', title: 'Apa kondisi kesehatan Anda?', hint: 'Tuliskan diagnosis, penyakit yang pernah dialami, obat yang dikonsumsi, dan akses terhadap bahan makanan. Jika sehat, tuliskan sehat.' },
  { id: 'BD', label: 'Biokimia', title: 'Apakah ada hasil laboratorium?', hint: 'Masukkan hasil gula darah, HbA1c, kolesterol, kreatinin, elektrolit, atau pemeriksaan lain bila tersedia.' },
  { id: 'PD', label: 'Fisik', title: 'Apakah ada keluhan fisik?', hint: 'Tuliskan edema, perubahan massa otot/lemak, gangguan menelan, atau tanda kekurangan gizi. Jika tidak ada, pilih tombol Tidak Ada.' },
];

const initialChat: ChatEntry[] = [{ id: 0, role: 'assistant', content: 'Halo, saya siap membantu menyusun rencana makan klinis. Kita mulai dengan asesmen singkat agar menu aman dan sesuai kebutuhan Anda.' }];
let nextChatId = 1;
const chatId = () => nextChatId++;

const formatAiError = (error: unknown) => {
  const value = error instanceof Error ? error.message : typeof error === 'string' ? error : error && typeof error === 'object' ? JSON.stringify(error) : '';
  let message = value || 'Kesalahan AI tidak diketahui';
  try {
    const parsed = JSON.parse(message) as { kind?: string; message?: string };
    if (parsed.message) message = `${parsed.kind ? `[${parsed.kind}] ` : ''}${parsed.message}`;
  } catch { }
  return `AI gagal: ${message.slice(0, 500)}`;
};

export default function AiMealPlanner({ meals, targets, clinicalContext, dashboardFoods, onImplement, onNotify }: { meals: MealTime[]; targets: Targets; clinicalContext: TdeeClinicalContext | null; dashboardFoods: SessionFood[]; onImplement: (rows: AiMealRow[]) => void; onNotify: (message: string) => void }) {
  const [config, setConfig] = useState<AiConfig>({ provider: 'openrouter', model: '', apiKey: '', baseUrl: '' });
  const [plannerClinicalContext, setPlannerClinicalContext] = useState<TdeeClinicalContext | null>(clinicalContext);
  const [defaultAvailable, setDefaultAvailable] = useState(false);
  const [rows, setRows] = useState<AiMealRow[]>([]);
  const [loading, setLoading] = useState(false);
  const [prompt, setPrompt] = useState('');
  const [chat, setChat] = useState<ChatEntry[]>(initialChat);
  const [assessmentAnswers, setAssessmentAnswers] = useState<AssessmentAnswer[]>([]);
  const [assessmentValue, setAssessmentValue] = useState('');
  const [assessmentVisible, setAssessmentVisible] = useState(true);
  const [assessmentExiting, setAssessmentExiting] = useState(false);
  const [planStarted, setPlanStarted] = useState(false);
  const [streamingId, setStreamingId] = useState<number | null>(null);
  const [sessionHydrated, setSessionHydrated] = useState(false);
  const chatHistoryRef = useRef<HTMLDivElement>(null);
  const requestInFlight = useRef(false);
  const requestGeneration = useRef(0);
  const activeRequestId = useRef<string | null>(null);
  const sessionState = useRef<PlannerSession>({ rows, chat, assessmentAnswers, assessmentVisible, planStarted, clinicalContext: plannerClinicalContext });

  useEffect(() => {
    if (clinicalContext) setPlannerClinicalContext(clinicalContext);
  }, [clinicalContext]);

  useEffect(() => {
    try {
      const saved = sessionStorage.getItem(storageKey);
      if (saved) {
        const state = JSON.parse(saved) as { rows?: AiMealRow[]; chat?: ChatEntry[]; assessmentAnswers?: AssessmentAnswer[]; assessmentVisible?: boolean; planStarted?: boolean; clinicalContext?: TdeeClinicalContext | null };
        if (state.rows) setRows(state.rows);
        if (state.chat?.length) setChat(state.chat.map(entry => ({ ...entry, id: chatId(), streaming: false })));
        if (state.assessmentAnswers) setAssessmentAnswers(state.assessmentAnswers);
        if (typeof state.assessmentVisible === 'boolean') setAssessmentVisible(state.assessmentVisible);
        if (typeof state.planStarted === 'boolean') setPlanStarted(state.planStarted);
        if (state.clinicalContext) setPlannerClinicalContext(state.clinicalContext);
      }
    } catch { }
    setSessionHydrated(true);
    try {
      const saved = localStorage.getItem('nutrisurvey.ai-preferences');
      if (saved) setConfig(current => ({ ...current, ...JSON.parse(saved) }));
    } catch { }
    loadAiKey().then(value => { if (value) setConfig(current => ({ ...current, apiKey: value })); }).catch(() => undefined);
    const onConfigUpdated = (event: Event) => setConfig((event as CustomEvent<AiConfig>).detail);
    window.addEventListener('nutrisurvey.ai-config-updated', onConfigUpdated);
    getAiDefaultInfo().then(info => {
      setDefaultAvailable(info.available);
      setConfig(current => current.provider === 'builtin_default' && info.available ? { ...current, model: info.model, baseUrl: info.baseUrl, apiKey: '' } : current);
    }).catch(() => undefined);
    return () => window.removeEventListener('nutrisurvey.ai-config-updated', onConfigUpdated);
  }, []);

  useEffect(() => {
    if (streamingId === null) return;
    const history = chatHistoryRef.current;
    if (history) history.scrollTo({ top: history.scrollHeight, behavior: 'auto' });
  }, [chat, streamingId]);

  useEffect(() => {
    if (!sessionHydrated) return;
    sessionState.current = { rows, chat, assessmentAnswers, assessmentVisible, planStarted, clinicalContext: plannerClinicalContext };
    try { sessionStorage.setItem(storageKey, JSON.stringify(sessionState.current)); } catch { }
  });

  const userMessages = useMemo(() => chat.filter(entry => entry.role === 'user'), [chat]);
  const estimatedContextTokens = useMemo(() => estimateTokens(chat.map(entry => `${entry.role}: ${entry.content}`).join('\n')), [chat]);
  const contextLimitReached = estimatedContextTokens >= maxContextTokens;
  const answeredSteps = assessmentAnswers.map(answer => answer.id);
  const assessmentComplete = assessmentAnswers.length >= assessmentSteps.length;
  const showChat = planStarted;
  const currentAssessment = assessmentSteps[assessmentAnswers.length];
  const providerLabel = providerNames[config.provider] || config.provider;
  const baseUrl = config.baseUrl || ({ openrouter: 'https://openrouter.ai/api/v1', openai: 'https://api.openai.com/v1', google: 'https://generativelanguage.googleapis.com/v1beta', anthropic: 'https://api.anthropic.com/v1' }[config.provider] || '');
  const effectiveTargets = plannerClinicalContext?.targets || targets;
  const clinicalSummary = plannerClinicalContext ? `Data klinis terverifikasi dari Kalkulator TDEE:\nJenis kelamin: ${plannerClinicalContext.request.gender}\nBerat badan: ${plannerClinicalContext.request.weightKg} kg\nTinggi badan: ${plannerClinicalContext.request.heightCm} cm\nUsia: ${plannerClinicalContext.request.age} tahun\nStandar IMT: ${plannerClinicalContext.request.bmiStandard}\nIMT: ${plannerClinicalContext.assessment.bmi}\nKlasifikasi gizi: ${plannerClinicalContext.assessment.nutritionClassification}\nBB ideal: ${plannerClinicalContext.assessment.idealWeight} kg\nBB adjusted: ${plannerClinicalContext.assessment.adjustedWeight} kg\nBB referensi: ${plannerClinicalContext.assessment.referenceWeight} kg\nBMR: ${plannerClinicalContext.assessment.basalMetabolicRate} kkal\nTDEE: ${plannerClinicalContext.assessment.totalDailyEnergyExpenditure} kkal\nFaktor aktivitas: ${plannerClinicalContext.request.activityFactor}\nFaktor cedera: ${plannerClinicalContext.request.injuryFactor}\nTarget makro: KH ${plannerClinicalContext.targets.carbs} g, protein ${plannerClinicalContext.targets.protein} g, lemak ${plannerClinicalContext.targets.fat} g.` : '';

  const streamAssistant = (token: string, entryId: number) => {
    const plainToken = token.replace(/```(?:json)?/gi, '').replace(/```/g, '');
    setChat(current => current.map(entry => entry.id === entryId ? { ...entry, content: entry.content === 'AI sedang menyusun...' ? plainToken : entry.content + plainToken, streaming: true } : entry));
  };

  const finishStream = (entryId: number, message: string, generation = requestGeneration.current) => {
    let index = 0;
    const timer = window.setInterval(() => {
      if (generation !== requestGeneration.current) { window.clearInterval(timer); return; }
      index = Math.min(message.length, index + Math.max(2, Math.ceil(message.length / 35)));
      setChat(current => current.map(entry => entry.id === entryId ? { ...entry, content: message.slice(0, index), streaming: true } : entry));
      if (index >= message.length) {
        window.clearInterval(timer);
        setChat(current => current.map(entry => entry.id === entryId ? { ...entry, content: message, streaming: false } : entry));
        setStreamingId(null);
      }
    }, 30);
  };

  const revealAssistant = (message: string) => {
    const entryId = chatId();
    setStreamingId(entryId);
    setChat(current => [...current, { id: entryId, role: 'assistant', content: '', streaming: true }]);
    let index = 0;
    const timer = window.setInterval(() => {
      index = Math.min(message.length, index + Math.max(2, Math.ceil(message.length / 45)));
      setChat(current => current.map(entry => entry.id === entryId ? { ...entry, content: message.slice(0, index) } : entry));
      if (index >= message.length) {
        window.clearInterval(timer);
        setChat(current => current.map(entry => entry.id === entryId ? { id: entryId, role: 'assistant', content: message } : entry));
        setStreamingId(null);
      }
    }, 28);
  };

  const answerAssessment = (value: string) => {
    if (!currentAssessment || !value.trim()) return;
    const answer = { id: currentAssessment.id, value: value.trim() };
    const nextAnswers = [...assessmentAnswers, answer];
    setAssessmentAnswers(nextAnswers);
    setAssessmentValue('');
    setChat(current => [...current, { id: chatId(), role: 'user', content: value.trim() }]);
    if (nextAnswers.length < assessmentSteps.length) {
      window.setTimeout(() => revealAssistant(`Terima kasih. ${assessmentSteps[nextAnswers.length].title} ${assessmentSteps[nextAnswers.length].hint}`), 180);
    } else {
      window.setTimeout(() => { setAssessmentExiting(true); window.setTimeout(() => { setAssessmentVisible(false); setAssessmentExiting(false); }, 300); }, 180);
    }
  };

  const generate = async (instruction: string, userAlreadyAdded = false) => {
    if (contextLimitReached || requestInFlight.current) return;
    requestInFlight.current = true;
    const generation = requestGeneration.current;
    stopRequestedRef.current = false;
    if (contextLimitReached) {
      requestInFlight.current = false;
      return;
    }
    if (!plannerClinicalContext || !config.model || (config.provider !== 'builtin_default' && !config.apiKey) || effectiveTargets.kcal <= 0) {
      const validationMessage = !plannerClinicalContext ? 'Hitung TDEE dan diagnosis gizi terlebih dahulu.' : 'Konfigurasi AI wajib disiapkan di Pengaturan.';
      setChat(current => [...current, { id: chatId(), role: 'assistant' as const, content: validationMessage, isError: true, retryPrompt: instruction }]);
      onNotify(`AI gagal: ${validationMessage}`);
      requestInFlight.current = false;
      return;
    }
    setLoading(true);
    if (!userAlreadyAdded) setChat(current => [...current, { id: chatId(), role: 'user' as const, content: instruction }]);
    const assessment = assessmentAnswers.map(answer => `${answer.id}: ${answer.value}`).join('\n');
    const isRevision = rows.length > 0;
    const activeMenu = rows.map(row => ({ ...row }));
    const dashboardContext = dashboardFoods.length > 0
      ? `\n\nKONSUMSI MAKANAN DI DASHBOARD SAAT INI (baca untuk perencanaan yang lebih baik):\n${meals.map(meal => {
          const mealFoods = dashboardFoods.filter(f => f.mealTime === meal.id);
          if (!mealFoods.length) return null;
          const lines = mealFoods.map(f => {
            const ratio = f.amount / (f.servingSize || 100);
            const kcal = ((f.nutrients as Record<string, number>)['Energi'] ?? (f.nutrients as Record<string, number>)['energy'] ?? 0) * ratio;
            const carbs = ((f.nutrients as Record<string, number>)['Karbohidrat total'] ?? (f.nutrients as Record<string, number>)['carbs'] ?? 0) * ratio;
            const protein = ((f.nutrients as Record<string, number>)['Protein'] ?? (f.nutrients as Record<string, number>)['protein'] ?? 0) * ratio;
            const fat = ((f.nutrients as Record<string, number>)['Lemak total'] ?? (f.nutrients as Record<string, number>)['fat'] ?? 0) * ratio;
            return `  - ${f.name}: ${f.amount}${f.servingUnit} | ${kcal.toFixed(0)} kcal | KH ${carbs.toFixed(1)}g | Protein ${protein.toFixed(1)}g | Lemak ${fat.toFixed(1)}g`;
          });
          return `[${meal.label}]\n${lines.join('\n')}`;
        }).filter(Boolean).join('\n')}`
      : '';
    const currentMenuSummary = isRevision
      ? `\n\nMENU SAAT INI (baca sebelum merencanakan revisi):\n${rows.map(row => `- [${row.meal_type}] ${row.matched_food_name || row.requested_keyword}: ${row.suggested_grams}g | ${row.calories.toFixed(0)} kcal | KH ${row.carbohydrate.toFixed(1)}g | Protein ${row.protein.toFixed(1)}g | Lemak ${row.fat.toFixed(1)}g`).join('\n')}`
      : '';
    const requestPrompt = `${clinicalSummary}\n\nAsesmen klinis tambahan pengguna:\n${assessment}${dashboardContext}${currentMenuSummary}\n\nInstruksi menu terbaru: ${instruction}\n\n${isRevision ? 'Ini adalah revisi menu aktif. Pertahankan item lama, ubah gram terlebih dahulu, tambah makanan hanya bila makro belum tercapai, dan hapus hanya jika diminta eksplisit.' : 'Ini adalah pembuatan menu awal.'}`;
    const streamId = chatId();
    const requestId = `ai-${chatId()}-${streamId}`;
    activeRequestId.current = requestId;
    setStreamingId(streamId);
    const errorId = chatId();
    pendingStopRef.current = { streamId, errorId, instruction };
    setChat(current => [...current, { id: streamId, role: 'assistant' as const, content: 'AI sedang menyusun...', streaming: true }]);
    try {
      const result = await streamAiMenu({ targetTDEE: Math.round(effectiveTargets.kcal), targetCarbs: Math.round(effectiveTargets.carbs), targetProtein: Math.round(effectiveTargets.protein), targetFat: Math.round(effectiveTargets.fat), prompt: requestPrompt, activeMenu, revision: isRevision, verifyMenu: true, requestId, availableMealTypes: meals.map(meal => meal.label), aiConfig: { ...config, baseUrl }, onToken: token => generation === requestGeneration.current && streamAssistant(token, streamId) });
       if (generation !== requestGeneration.current) return;
       pendingStopRef.current = null;
       setChat(current => current.filter(entry => entry.id !== errorId));
       setRows(result);
       finishStream(streamId, `Rencana makan selesai dibuat dengan ${result.length} item. Tinjau panel Rencana Menu, lalu tekan Terapkan ke Dashboard jika sudah sesuai.`, generation);
      setPrompt('');
    } catch (cause) {
      if (generation !== requestGeneration.current) return;
        const message = formatAiError(cause);
        onNotify(message);
        setChat(current => [...current.filter(entry => entry.id !== streamId), { id: errorId, role: 'assistant' as const, content: message, isError: true, retryPrompt: instruction }]);
        setStreamingId(null);
        if (/NotAllowed|not found|channel|IPC|invoke|request failed|response could not be read|decoding response body|body read|network|timeout/i.test(message)) {
          try {
            const result = await generateAiMenu({ targetTDEE: Math.round(effectiveTargets.kcal), targetCarbs: Math.round(effectiveTargets.carbs), targetProtein: Math.round(effectiveTargets.protein), targetFat: Math.round(effectiveTargets.fat), prompt: requestPrompt, activeMenu, revision: isRevision, verifyMenu: true, requestId, availableMealTypes: meals.map(meal => meal.label), aiConfig: { ...config, baseUrl } });
            if (generation !== requestGeneration.current) return;
            setRows(result);
            setChat(current => current.map(entry => entry.id === errorId ? { ...entry, content: 'AI sedang menyusun...', isError: false, retryPrompt: undefined, streaming: true } : entry));
            finishStream(errorId, `Rencana makan selesai dibuat dengan ${result.length} item. Tinjau panel Rencana Menu, lalu tekan Terapkan ke Dashboard.`, generation);
            return;
          } catch (fallbackCause) {
            const retryMessage = formatAiError(fallbackCause);
            setChat(current => current.map(entry => entry.id === errorId ? { ...entry, content: retryMessage, isError: true, retryPrompt: instruction } : entry));
           }
         }
    } finally {
      pendingStopRef.current = null;
      if (generation === requestGeneration.current) {
        requestInFlight.current = false;
        activeRequestId.current = null;
        setLoading(false);
      }
    }
  };

  const startNewChat = () => {
    if (activeRequestId.current) void cancelAiRequest(activeRequestId.current);
    activeRequestId.current = null;
    requestGeneration.current += 1;
    requestInFlight.current = false;
    try { sessionStorage.removeItem(storageKey); } catch { }
    setLoading(false);
    setRows([]);
    setChat([{ id: chatId(), role: 'assistant', content: 'Halo, saya siap membantu menyusun rencana makan klinis. Kita mulai dengan asesmen singkat agar menu aman dan sesuai kebutuhan Anda.' }]);
    setAssessmentAnswers([]);
    setAssessmentValue('');
    setAssessmentVisible(true);
    setAssessmentExiting(false);
    setPlanStarted(false);
    setPrompt('');
    setStreamingId(null);
  };

  const retryMessage = async (entry: ChatEntry) => {
    if (!entry.retryPrompt || requestInFlight.current || loading || streamingId !== null) return;
    setChat(current => current.filter(item => item.id !== entry.id));
    await generate(entry.retryPrompt, true);
  };

  const startPlan = async () => {
    if (!assessmentComplete || loading || streamingId !== null) return;
    setPlanStarted(true);
    await generate(planRequest);
  };

  const stopRequestedRef = useRef(false);
  const pendingStopRef = useRef<{ streamId: number; errorId: number; instruction: string } | null>(null);

  const stopGeneration = () => {
    stopRequestedRef.current = true;
    if (pendingStopRef.current) {
      const { streamId, errorId, instruction } = pendingStopRef.current;
      pendingStopRef.current = null;
      setChat(current => [...current.filter(entry => entry.id !== streamId), { id: errorId, role: 'assistant' as const, content: 'Permintaan dihentikan', isError: true, retryPrompt: instruction }]);
      setStreamingId(null);
    }
    requestGeneration.current += 1;
    if (activeRequestId.current) void cancelAiRequest(activeRequestId.current);
    activeRequestId.current = null;
    requestInFlight.current = false;
    setLoading(false);
  };

  const sendMessage = async () => {
    if (loading) {
      stopGeneration();
      return;
    }
    const message = prompt.trim();
    if (!message || !planStarted || streamingId !== null) return;
    setPrompt('');
    await generate(message);
  };

  return <section className="ai-planner-page">
    <div className="section-header ai-planner-heading"><div><span className="eyebrow">Clinical nutrition workspace</span><h2>AI Meal Planner</h2><p>Susun rencana makan personal melalui asesmen gizi berbasis PAGT.</p></div><div className="ai-provider-status"><i />{providerLabel}{defaultAvailable && config.provider === 'builtin_default' && <span>ENV Default</span>}</div></div>
    <div className="ai-clinical-strip">{plannerClinicalContext ? <><div className="ai-clinical-row diagnosis"><div className="ai-clinical-cell"><small>Diagnosis Gizi</small><b>{plannerClinicalContext.assessment.nutritionClassification}</b></div><div className="ai-clinical-cell"><small>IMT</small><b>{plannerClinicalContext.assessment.bmi.toFixed(1)}</b></div><div className="ai-clinical-cell"><small>TDEE</small><b>{Math.round(plannerClinicalContext.assessment.totalDailyEnergyExpenditure).toLocaleString('id-ID')} kkal</b></div><div className="ai-clinical-cell"><small>BB Referensi</small><b>{plannerClinicalContext.assessment.referenceWeight.toFixed(1)} kg</b></div></div><div className="ai-clinical-row macros"><div className="ai-clinical-cell"><small>Target Energi</small><b>{Math.round(effectiveTargets.kcal).toLocaleString('id-ID')} <em>kkal</em></b></div><div className="ai-clinical-cell"><small>Protein</small><b>{Math.round(effectiveTargets.protein)} <em>g</em></b></div><div className="ai-clinical-cell"><small>Karbohidrat</small><b>{Math.round(effectiveTargets.carbs)} <em>g</em></b></div><div className="ai-clinical-cell"><small>Lemak</small><b>{Math.round(effectiveTargets.fat)} <em>g</em></b></div></div></> : <div className="ai-clinical-missing"><b>Data klinis belum tersedia</b><span>Hitung TDEE terlebih dahulu.</span></div>}</div>
    <div className="ai-planner-layout">
      <div className="card ai-plan-panel"><div className="ai-panel-header"><div><span className="eyebrow">AI generated plan</span><h3>Rencana Menu</h3></div>{rows.length > 0 && <button className="primary" onClick={() => onImplement(rows)}>Terapkan ke Dashboard</button>}</div>{rows.length === 0 ? <div className="ai-plan-empty"><div className="ai-empty-mark">✦</div><h4>Rencana menu akan muncul di sini</h4><p>Lengkapi asesmen dan minta AI membuat menu. Anda dapat meninjau hasil sebelum menerapkannya.</p></div> : <div className="ai-plan-list">{rows.map((row, index) => <div className="ai-plan-item" key={`${row.meal_type}-${row.matched_food_id}-${index}`}><div className="ai-plan-item-top"><div><small>{row.meal_type}</small><h4>{row.matched_food_name || row.requested_keyword}</h4></div><strong>{row.suggested_grams} g</strong></div><div className="ai-plan-item-meta"><span>{row.calories.toFixed(0)} kkal</span><span>Protein {row.protein.toFixed(1)} g</span><span>KH {row.carbohydrate.toFixed(1)} g</span><span>Lemak {row.fat.toFixed(1)} g</span></div><p>{row.reasoning}</p></div>)}</div>}{rows.length > 0 && <div className="ai-disclaimer">Rekomendasi bersifat edukatif dan bukan pengganti konsultasi dokter atau ahli gizi teregistrasi.</div>}</div>
      <div className="card ai-conversation-panel"><div className="ai-chat-header"><div><span className="eyebrow">Personal dietitian</span><h3>Chat dengan AI</h3><p>Asesmen klinis sebelum rekomendasi menu</p></div><div className="ai-chat-header-actions"><button className="new-chat-btn" onClick={startNewChat}>+ New Chat</button><span className="ai-online"><i /> Online</span></div></div><div className="ai-assessment-progress">{assessmentSteps.map(step => <span className={answeredSteps.includes(step.id) ? 'done' : ''} key={step.id}><b>{step.id}</b>{step.label}</span>)}</div>{assessmentVisible && !assessmentComplete && currentAssessment && <div className={`ai-assessment-card ${assessmentExiting ? 'assessment-exiting' : ''}`}><div className="ai-assessment-card-head"><span>Langkah {assessmentAnswers.length + 1} dari {assessmentSteps.length}</span><b>{currentAssessment.id}</b></div><h4>{currentAssessment.title}</h4><p>{currentAssessment.hint}</p><textarea value={assessmentValue} onChange={event => setAssessmentValue(event.target.value)} placeholder="Tulis jawaban Anda..." rows={3} /><div className="ai-assessment-actions"><button className="secondary" onClick={() => answerAssessment('Tidak ada')}>Tidak Ada</button><button className="primary" disabled={!assessmentValue.trim()} onClick={() => answerAssessment(assessmentValue)}>Simpan Jawaban</button></div></div>}{assessmentComplete && !planStarted && <div className="ai-plan-cta"><div><b>Asesmen selesai</b><span>AI siap menyusun plan menu berdasarkan jawaban Anda.</span></div><button className="primary" disabled={!plannerClinicalContext} onClick={() => void startPlan()}>Buat Plan Menu Sekarang <span>↗</span></button></div>}{showChat && <><div className="ai-chat-history" ref={chatHistoryRef}>{chat.map((entry, index) => <div className={`ai-chat-message ${entry.role} ${entry.streaming ? 'message-streaming' : ''}`} key={entry.id}><div className="ai-chat-avatar">{entry.role === 'assistant' ? 'AI' : 'Anda'}</div><div className={`ai-chat-bubble ${entry.isError ? 'ai-error-bubble' : ''}`}>{entry.streaming && entry.content === 'AI sedang menyusun...' ? <div className="ai-streaming-status"><span>AI sedang menyusun</span><i /><i /><i /></div> : <p>{entry.content}{entry.streaming && <span className="stream-caret" />}</p>}{entry.isError && <button className="ai-retry-btn" aria-label="Ulangi permintaan" title="Ulangi permintaan" onClick={event => { event.stopPropagation(); void retryMessage(entry); }} disabled={loading || streamingId !== null}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20 11a8.1 8.1 0 0 0-15.5-2M4 5v4h4M4 13a8.1 8.1 0 0 0 15.5 2M20 19v-4h-4" /></svg></button>}</div></div>)}</div><div className="ai-chat-compose"><textarea value={prompt} onChange={event => setPrompt(event.target.value)} onKeyDown={event => { if (event.key === 'Enter' && !event.shiftKey) { event.preventDefault(); void sendMessage(); } }} placeholder="Tulis revisi atau perubahan pada plan menu..." rows={3} disabled={!assessmentComplete || contextLimitReached} /><button className="primary ai-send-btn" aria-label={loading ? 'Hentikan respons AI' : 'Kirim'} title={loading ? 'Hentikan respons AI' : 'Kirim'} disabled={!loading && (!prompt.trim() || !assessmentComplete)} onClick={() => void sendMessage()}>{loading ? <span aria-hidden="true">■</span> : <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-label="Kirim"><line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/></svg>}</button></div></>}{contextLimitReached && <div className="ai-context-limit">Kuota Melebihi Batas, Tolong Buat Percakapan Baru</div>}</div>
    </div>
  </section>;
}
