import { type FormEvent, useEffect, useMemo, useState } from 'react';
import type { WorkspacePlan } from '@kidos/contracts';
import type { KidOSApi } from '../../lib/kidos-api';
import { prepareProtectedNavigation } from '../browser/protected-navigation';

type ActiveTool = 'Notes' | 'Calculator' | 'KidOS AI' | 'Math Lab' | 'Science Explorer' | 'Reading Room' | 'Music' | 'Puzzle Park' | 'Racing' | 'Strategy' | 'Explore' | 'Animals' | 'Space' | 'Files' | 'Camera' | 'Messages' | 'Screen Time' | 'Accessibility' | 'Guardian' | null;

type Section =
  | 'Home'
  | 'Learn'
  | 'Play'
  | 'Create'
  | 'Watch'
  | 'Search'
  | 'My Apps'
  | 'KidOS AI'
  | 'Messages'
  | 'Settings';

const navItems: Array<{ label: Section; icon: string }> = [
  { label: 'Home', icon: '⌂' },
  { label: 'Learn', icon: '📘' },
  { label: 'Play', icon: '🎮' },
  { label: 'Create', icon: '🎨' },
  { label: 'Watch', icon: '▶' },
  { label: 'Search', icon: '⌕' },
  { label: 'My Apps', icon: '▦' },
  { label: 'KidOS AI', icon: '✦' },
  { label: 'Messages', icon: '✉' },
  { label: 'Settings', icon: '⚙' },
];

const cards: Array<{ section: Section; icon: string; title: string; copy: string; tone: string }> = [
  { section: 'Learn', icon: '📗', title: 'Learn', copy: 'Math • Science • Reading', tone: 'blue' },
  { section: 'Play', icon: '🎮', title: 'Play', copy: 'Fun & safe games', tone: 'purple' },
  { section: 'Create', icon: '🖌️', title: 'Create', copy: 'Draw • Music • Build', tone: 'orange' },
  { section: 'Watch', icon: '▶️', title: 'Watch', copy: 'Videos picked for you', tone: 'red' },
  { section: 'Search', icon: '🔎', title: 'Safe Search', copy: 'Explore the web safely', tone: 'green' },
  { section: 'My Apps', icon: '▦', title: 'My Apps', copy: 'Your approved apps', tone: 'yellow' },
  { section: 'KidOS AI', icon: '🤖', title: 'KidOS AI', copy: 'Ask me anything', tone: 'sky' },
  { section: 'Settings', icon: '🛡️', title: 'Wellbeing', copy: 'Screen time & balance', tone: 'mint' },
];

const moduleCards: Partial<Record<Section, Array<{ icon: string; title: string; copy: string }>>> = {
  Learn: [
    { icon: '➗', title: 'Math Lab', copy: 'Practice numbers, puzzles, and problem solving.' },
    { icon: '🔬', title: 'Science Explorer', copy: 'Discover nature, space, and experiments.' },
    { icon: '📚', title: 'Reading Room', copy: 'Build reading skills with approved stories.' },
  ],
  Play: [
    { icon: '🧩', title: 'Puzzle Park', copy: 'Brain games selected for your profile.' },
    { icon: '🏎️', title: 'Racing', copy: 'Kid-friendly racing and reflex games.' },
    { icon: '♟️', title: 'Strategy', copy: 'Think ahead with safe strategy games.' },
  ],
  Watch: [
    { icon: '🌎', title: 'Explore', copy: 'Curated educational videos about our world.' },
    { icon: '🐾', title: 'Animals', copy: 'Nature and wildlife videos that pass KidOS checks.' },
    { icon: '🚀', title: 'Space', copy: 'Approved science and astronomy videos.' },
  ],
  'My Apps': [
    { icon: '📝', title: 'Notes', copy: 'Write ideas, homework, and reminders.' },
    { icon: '🧮', title: 'Calculator', copy: 'A simple calculator for schoolwork.' },
    { icon: '🎵', title: 'Music', copy: 'Open approved music tools and content.' },
  ],
  'KidOS AI': [
    { icon: '💡', title: 'Ask a Question', copy: 'Get age-appropriate help with learning.' },
    { icon: '✨', title: 'Create an Idea', copy: 'Brainstorm stories, projects, and drawings.' },
    { icon: '🛡️', title: 'Protected Answers', copy: 'Responses stay inside KidOS safety rules.' },
  ],
  Settings: [
    { icon: '⏱️', title: 'Screen Time', copy: 'See your daily balance and break schedule.' },
    { icon: '♿', title: 'Accessibility', copy: 'Adjust text, motion, sound, and display.' },
    { icon: '🔒', title: 'Guardian', copy: 'Parent-controlled safety settings stay protected.' },
  ],
};

const sectionDescriptions: Record<Exclude<Section, 'Home' | 'Create' | 'Search'>, string> = {
  Learn: 'Your approved learning tools and school-friendly activities will live here.',
  Play: 'Only games approved for your KidOS profile will appear here.',
  Watch: 'KidOS will show videos that pass your profile and media-safety rules.',
  'My Apps': 'This is your personal shelf of approved KidOS apps.',
  'KidOS AI': 'Your protected AI helper for questions, learning, and creative ideas.',
  Messages: 'Safe family and approved-contact messages will appear here.',
  Settings: 'Profile, accessibility, sound, display, and wellbeing controls will live here.',
};

function workspaceLabel(kind: WorkspacePlan['kind']) {
  switch (kind) {
    case 'story':
      return 'Story workspace';
    case 'drawing_presentation':
      return 'Draw & Present workspace';
    case 'beginner_coding':
      return 'Beginner Coding workspace';
  }
}

export default function ChildHome({ api }: { api: KidOSApi }) {
  const [activeSection, setActiveSection] = useState<Section>('Home');
  const [prompt, setPrompt] = useState('');
  const [plan, setPlan] = useState<WorkspacePlan | null>(null);
  const [url, setUrl] = useState('');
  const [navigationState, setNavigationState] = useState<string | null>(null);
  const [demoStep, setDemoStep] = useState(0);
  const [activeTool, setActiveTool] = useState<ActiveTool>(null);
  const [noteText, setNoteText] = useState('');
  const [calcInput, setCalcInput] = useState('');
  const [calcResult, setCalcResult] = useState('');
  const [aiQuestion, setAiQuestion] = useState('');
  const [aiAnswer, setAiAnswer] = useState('Ask me a school-safe question and I’ll help you think it through.');
  const [puzzleScore, setPuzzleScore] = useState(0);
  const [raceStartedAt, setRaceStartedAt] = useState<number | null>(null);
  const [raceResult, setRaceResult] = useState('Tap Start, then tap Finish as quickly as you can.');
  const [strategyBoard, setStrategyBoard] = useState<Array<'X' | 'O' | null>>(Array(9).fill(null));
  const [selectedVideo, setSelectedVideo] = useState<string | null>(null);
  const [files, setFiles] = useState(['Homework Ideas.txt', 'Space Story.kidos', 'Reading Notes.txt']);
  const [newFileName, setNewFileName] = useState('');
  const [cameraStatus, setCameraStatus] = useState('Camera is off. KidOS asks before using it.');
  const [messageText, setMessageText] = useState('');
  const [messages, setMessages] = useState(['Parent: Have a great learning day!']);
  const [largeText, setLargeText] = useState(false);
  const [reducedMotion, setReducedMotion] = useState(false);

  const demoMode = useMemo(
    () =>
      typeof window !== 'undefined' &&
      new URLSearchParams(window.location.search).get('demo') === '1',
    [],
  );

  const demoSequence = useMemo<Section[]>(
    () => ['Home', 'Learn', 'Play', 'Create', 'Search', 'KidOS AI', 'Settings', 'Home'],
    [],
  );

  const currentDate = useMemo(
    () =>
      new Intl.DateTimeFormat('en-US', {
        weekday: 'long',
        month: 'short',
        day: 'numeric',
      }).format(new Date()),
    [],
  );

  useEffect(() => {
    if (!demoMode) return;

    const applyDemoStep = (step: number) => {
      const section = demoSequence[step] ?? 'Home';
      setActiveSection(section);
      setPlan(null);

      if (section === 'Create') {
        setPrompt('Make a space story with friendly robots');
      } else {
        setPrompt('');
      }

      if (section === 'Search') {
        setUrl('https://blocked.example');
        setNavigationState('Site blocked by KidOS');
      } else {
        setUrl('');
        setNavigationState(null);
      }
    };

    applyDemoStep(0);
    const interval = window.setInterval(() => {
      setDemoStep((current) => {
        const next = (current + 1) % demoSequence.length;
        applyDemoStep(next);
        return next;
      });
    }, 2800);

    return () => window.clearInterval(interval);
  }, [demoMode, demoSequence]);

  async function createWorkspace(event: FormEvent) {
    event.preventDefault();
    const request = prompt.trim();
    if (!request) return;
    setPlan(await api.planWorkspace(request));
  }

  async function checkSite(event: FormEvent) {
    event.preventDefault();
    const destination = url.trim();
    if (!destination) return;

    const result = await prepareProtectedNavigation(destination, (checkedUrl) =>
      api.evaluateNavigation(checkedUrl),
    );

    if (result.state === 'load') {
      setNavigationState(`Opening ${result.url}`);
    } else if (result.state === 'blocked') {
      setNavigationState('Site blocked by KidOS');
    } else {
      setNavigationState('Parent approval required');
    }
  }

  function openSection(section: Section) {
    setActiveTool(null);
    setActiveSection(section);
    setPlan(null);
    setNavigationState(null);
  }


  function openTool(title: string) {
    const supported: ActiveTool[] = ['Notes', 'Calculator', 'KidOS AI', 'Math Lab', 'Science Explorer', 'Reading Room', 'Music', 'Puzzle Park', 'Racing', 'Strategy', 'Explore', 'Animals', 'Space', 'Files', 'Camera', 'Messages', 'Screen Time', 'Accessibility', 'Guardian'];
    if (supported.includes(title as ActiveTool)) {
      setActiveTool(title as ActiveTool);
    }
  }

  function calculate() {
    const match = calcInput.trim().match(/^(-?\d+(?:\.\d+)?)\s*([+\-*/])\s*(-?\d+(?:\.\d+)?)$/);
    if (!match) {
      setCalcResult('Try something like 12 + 8');
      return;
    }
    const left = Number(match[1]);
    const right = Number(match[3]);
    const operator = match[2];
    if (operator === '/' && right === 0) {
      setCalcResult('Division by zero is not allowed.');
      return;
    }
    const value =
      operator === '+' ? left + right :
      operator === '-' ? left - right :
      operator === '*' ? left * right :
      left / right;
    setCalcResult(String(Number(value.toFixed(8))));
  }

  function answerKidOSAI(event: FormEvent) {
    event.preventDefault();
    const question = aiQuestion.trim();
    if (!question) return;
    const lower = question.toLowerCase();
    if (lower.includes('planet') || lower.includes('space')) {
      setAiAnswer('A planet is a large world that moves around a star. Earth is the planet we live on, and our solar system has eight planets.');
    } else if (lower.includes('math') || /\d/.test(question)) {
      setAiAnswer('I can help with that. Try breaking the problem into smaller steps, then check each step before moving on.');
    } else if (lower.includes('story') || lower.includes('write')) {
      setAiAnswer('Start with a character, give them a goal, add one challenge, and decide how they change by the end.');
    } else {
      setAiAnswer('That is a good question. KidOS AI would answer using age-appropriate, parent-approved sources and explain it in simple steps.');
    }
  }

  function markPuzzle(correct: boolean) {
    if (correct) setPuzzleScore((score) => score + 1);
  }

  function startRace() {
    setRaceStartedAt(Date.now());
    setRaceResult('Go! Tap Finish now.');
  }

  function finishRace() {
    if (!raceStartedAt) {
      setRaceResult('Tap Start first.');
      return;
    }
    const elapsed = (Date.now() - raceStartedAt) / 1000;
    setRaceResult(`Reaction time: ${elapsed.toFixed(2)} seconds`);
    setRaceStartedAt(null);
  }

  function playStrategy(index: number) {
    setStrategyBoard((board) => {
      if (board[index]) return board;
      const next = [...board];
      next[index] = 'X';
      const open = next.map((cell, i) => cell ? -1 : i).filter((i) => i >= 0);
      if (open.length) next[open[0]] = 'O';
      return next;
    });
  }

  function resetStrategy() {
    setStrategyBoard(Array(9).fill(null));
  }

  function addFile(event: FormEvent) {
    event.preventDefault();
    const name = newFileName.trim();
    if (!name) return;
    setFiles((current) => [...current, name]);
    setNewFileName('');
  }

  async function startCamera() {
    if (!navigator.mediaDevices?.getUserMedia) {
      setCameraStatus('Camera access is not available in this environment.');
      return;
    }
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: false });
      stream.getTracks().forEach((track) => track.stop());
      setCameraStatus('Camera permission confirmed. KidOS would now open the protected camera preview.');
    } catch {
      setCameraStatus('Camera permission was not granted.');
    }
  }

  function sendMessage(event: FormEvent) {
    event.preventDefault();
    const value = messageText.trim();
    if (!value) return;
    setMessages((current) => [...current, `You → Parent: ${value}`]);
    setMessageText('');
  }

  const toolPanel = activeTool ? (
    <section className="tool-window" aria-label={activeTool}>
      <div className="tool-window-header">
        <div>
          <p className="eyebrow">KidOS App</p>
          <h1>{activeTool}</h1>
        </div>
        <button type="button" className="tool-close" onClick={() => setActiveTool(null)}>Back</button>
      </div>

      {activeTool === 'Notes' ? (
        <div className="notes-tool">
          <textarea
            aria-label="KidOS Notes"
            value={noteText}
            onChange={(event) => setNoteText(event.target.value)}
            placeholder="Write your ideas, homework, or reminders here..."
          />
          <div className="tool-status">Saved in this KidOS session • {noteText.length} characters</div>
        </div>
      ) : activeTool === 'Calculator' ? (
        <div className="calculator-tool">
          <input
            aria-label="Calculator expression"
            value={calcInput}
            onChange={(event) => setCalcInput(event.target.value)}
            placeholder="12 + 8"
          />
          <button type="button" onClick={calculate}>Calculate</button>
          <output className="calculator-result">{calcResult || '0'}</output>
        </div>
      ) : activeTool === 'KidOS AI' ? (
        <div className="ai-tool">
          <div className="ai-answer">🤖 {aiAnswer}</div>
          <form onSubmit={answerKidOSAI}>
            <input
              aria-label="Ask KidOS AI"
              value={aiQuestion}
              onChange={(event) => setAiQuestion(event.target.value)}
              placeholder="Why is the sky blue?"
            />
            <button type="submit">Ask</button>
          </form>
        </div>
      ) : activeTool === 'Puzzle Park' ? (
        <div className="learning-tool">
          <div className="learning-score">🧩 Score: {puzzleScore}</div>
          <h2>Which number completes the pattern?</h2>
          <p>2, 4, 6, 8, ?</p>
          <div className="answer-row">
            <button type="button" onClick={() => markPuzzle(false)}>9</button>
            <button type="button" onClick={() => markPuzzle(true)}>10</button>
            <button type="button" onClick={() => markPuzzle(false)}>12</button>
          </div>
        </div>
      ) : activeTool === 'Racing' ? (
        <div className="learning-tool">
          <div className="learning-score">🏎️ Reflex Racer</div>
          <h2>Reaction challenge</h2>
          <p>{raceResult}</p>
          <div className="answer-row">
            <button type="button" onClick={startRace}>Start</button>
            <button type="button" onClick={finishRace}>Finish</button>
          </div>
        </div>
      ) : activeTool === 'Strategy' ? (
        <div className="learning-tool">
          <div className="learning-score">♟️ Strategy Board</div>
          <h2>Mini strategy game</h2>
          <div className="strategy-board">
            {strategyBoard.map((cell, index) => (
              <button type="button" key={index} onClick={() => playStrategy(index)}>{cell ?? ''}</button>
            ))}
          </div>
          <button type="button" className="primary-button" onClick={resetStrategy}>Reset board</button>
        </div>
      ) : activeTool === 'Explore' || activeTool === 'Animals' || activeTool === 'Space' ? (
        <div className="learning-tool">
          <div className="learning-score">▶ KidOS Watch • Approved</div>
          <h2>{activeTool}</h2>
          <div className="watch-grid">
            {[
              activeTool === 'Explore' ? 'Amazing Places on Earth' : activeTool === 'Animals' ? 'Wildlife Around the World' : 'Tour of the Solar System',
              activeTool === 'Explore' ? 'How Maps Work' : activeTool === 'Animals' ? 'How Animals Adapt' : 'Why Stars Shine',
              activeTool === 'Explore' ? 'Oceans and Mountains' : activeTool === 'Animals' ? 'Animal Habitats' : 'Moon Mission Basics',
            ].map((title) => (
              <button type="button" className="watch-card" key={title} onClick={() => setSelectedVideo(title)}>
                <span>▶</span><strong>{title}</strong><small>KidOS approved</small>
              </button>
            ))}
          </div>
          {selectedVideo ? <div className="now-playing">Now playing preview: <strong>{selectedVideo}</strong><br/>Educational media would stream only after KidOS safety checks.</div> : null}
        </div>
      ) : activeTool === 'Files' ? (
        <div className="learning-tool">
          <div className="learning-score">📁 My Files</div>
          <h2>Approved KidOS files</h2>
          <div className="file-list">
            {files.map((file) => <div className="file-row" key={file}><span>📄</span><strong>{file}</strong></div>)}
          </div>
          <form className="creation-bar" onSubmit={addFile}>
            <input aria-label="New file name" value={newFileName} onChange={(event) => setNewFileName(event.target.value)} placeholder="New file name" />
            <button type="submit">Add file</button>
          </form>
        </div>
      ) : activeTool === 'Camera' ? (
        <div className="learning-tool">
          <div className="learning-score">📷 Protected Camera</div>
          <h2>Camera access</h2>
          <p>{cameraStatus}</p>
          <button type="button" className="primary-button" onClick={startCamera}>Request camera access</button>
        </div>
      ) : activeTool === 'Messages' ? (
        <div className="learning-tool">
          <div className="learning-score">✉ Approved Messages</div>
          <h2>Family messages</h2>
          <div className="message-list">{messages.map((message, index) => <div className="message-bubble" key={index}>{message}</div>)}</div>
          <form className="creation-bar" onSubmit={sendMessage}>
            <input aria-label="Message parent" value={messageText} onChange={(event) => setMessageText(event.target.value)} placeholder="Message parent..." />
            <button type="submit">Send</button>
          </form>
        </div>
      ) : activeTool === 'Screen Time' ? (
        <div className="learning-tool">
          <div className="learning-score">⏱️ Wellbeing</div>
          <h2>Today’s screen time</h2>
          <div className="screen-time-ring">1h 24m</div>
          <p>45 minutes of learning • 24 minutes creating • 15 minutes playing.</p>
          <div className="fact-card">Next suggested break in 16 minutes.</div>
        </div>
      ) : activeTool === 'Accessibility' ? (
        <div className="learning-tool">
          <div className="learning-score">♿ Accessibility</div>
          <h2>Make KidOS comfortable</h2>
          <label className="toggle-row"><input type="checkbox" checked={largeText} onChange={(event) => setLargeText(event.target.checked)} /> Larger text</label>
          <label className="toggle-row"><input type="checkbox" checked={reducedMotion} onChange={(event) => setReducedMotion(event.target.checked)} /> Reduce motion</label>
          <div className="fact-card">These settings are saved for this KidOS session.</div>
        </div>
      ) : activeTool === 'Guardian' ? (
        <div className="learning-tool">
          <div className="learning-score">🔒 Guardian Protected</div>
          <h2>Parent controls are locked</h2>
          <p>Children can see that Guardian is active, but changing safety rules requires parent authorization.</p>
          <div className="fact-card">🛡 Safe Mode: ON • Web policy active • Downloads protected • Parent PIN required for changes.</div>
        </div>
      ) : activeTool === 'Math Lab' ? (
        <div className="learning-tool">
          <div className="learning-score">⭐ Quick Challenge</div>
          <h2>What is 7 × 8?</h2>
          <div className="answer-row">
            <button type="button">48</button><button type="button">54</button><button type="button">56</button>
          </div>
          <p>Choose an answer, then explain how you worked it out.</p>
        </div>
      ) : activeTool === 'Science Explorer' ? (
        <div className="learning-tool">
          <div className="learning-score">🔬 Science Explorer</div>
          <h2>Why do plants need sunlight?</h2>
          <p>Plants use light energy to make food in a process called photosynthesis. That energy helps them grow.</p>
          <div className="fact-card">🌱 Try this: compare a plant near a sunny window with one kept in a darker place.</div>
        </div>
      ) : activeTool === 'Reading Room' ? (
        <div className="learning-tool">
          <div className="learning-score">📚 Reading Room</div>
          <h2>The Curious Robot</h2>
          <p>A small robot looked at the night sky and wondered why the stars seemed to sparkle. It decided to learn one new thing every day.</p>
          <div className="fact-card">Question: What made the robot curious?</div>
        </div>
      ) : (
        <div className="learning-tool">
          <div className="learning-score">🎵 Music</div>
          <h2>Approved Music Space</h2>
          <p>KidOS can provide parent-approved music, simple rhythm activities, and creative audio tools here.</p>
          <button type="button" className="primary-button">Start Rhythm Activity</button>
        </div>
      )}
    </section>
  ) : null;

  if (plan) {
    return (
      <main className="kidos-shell concept-shell">
      {demoMode ? (
        <div className="demo-badge" role="status">
          <span className="demo-dot" />
          KidOS Demo • {demoSequence[demoStep]}
        </div>
      ) : null}
        <section className="workspace-screen">
          <span className="workspace-badge">Safe workspace ready</span>
          <div className="workspace-icon">✨</div>
          <h1>{plan.title}</h1>
          <p>{workspaceLabel(plan.kind)}</p>
          <button className="primary-button" type="button" onClick={() => setPlan(null)}>
            Back home
          </button>
        </section>
      </main>
    );
  }

  return (
    <main className="kidos-shell concept-shell">
      {demoMode ? (
        <div className="demo-badge" role="status">
          <span className="demo-dot" />
          KidOS Demo • {demoSequence[demoStep]}
        </div>
      ) : null}
      <aside className="sidebar" aria-label="KidOS navigation">
        <div className="brand">
          <div className="brand-mark">🐻</div>
          <div>
            <strong>KidOS</strong>
            <span>A Safer, Brighter Tomorrow</span>
          </div>
        </div>

        <nav className="side-nav">
          {navItems.map((item) => (
            <button
              type="button"
              key={item.label}
              className={activeSection === item.label ? 'nav-item active' : 'nav-item'}
              onClick={() => openSection(item.label)}
            >
              <span>{item.icon}</span>
              {item.label}
            </button>
          ))}
        </nav>

        <div className="profile-card">
          <div className="avatar">🧑🏽</div>
          <div>
            <strong>Alex</strong>
            <span>Explorer • Level 12</span>
          </div>
          <span className="profile-more">•••</span>
        </div>
      </aside>

      <section className="desktop">
        <header className="desktop-topbar">
          <label className="global-search">
            <span>⌕</span>
            <input
              aria-label="Search KidOS"
              placeholder="Search KidOS..."
              onFocus={() => activeSection === 'Home' && setActiveSection('Search')}
            />
          </label>

          <div className="status-cluster" aria-label="KidOS status">
            <span>🔔</span>
            <span className="safe-status">🛡 Safe Mode: ON</span>
            <span>📶</span>
            <span>🔋 92%</span>
            <span className="date-block">{currentDate}</span>
          </div>
        </header>

        <div className="scenery" aria-hidden="true">
          <div className="sun" />
          <div className="mountain mountain-one" />
          <div className="mountain mountain-two" />
          <div className="hill hill-one" />
          <div className="hill hill-two" />
          <div className="lake" />
          <div className="treehouse">🏡</div>
        </div>

        <div className="desktop-content">
          {toolPanel ? toolPanel : activeSection === 'Home' ? (
            <>
              <section className="welcome-row">
                <div>
                  <p className="eyebrow">Protected profile</p>
                  <h1>Good Morning, Alex! 👋</h1>
                  <p className="welcome-copy">What would you like to do today?</p>
                </div>
                <div className="guardian-chip">🛡 Guardian healthy</div>
              </section>

              <section className="app-grid" aria-label="KidOS apps">
                {cards.map((card) => (
                  <button
                    type="button"
                    key={card.title}
                    className={`app-card ${card.tone}`}
                    onClick={() => openSection(card.section)}
                  >
                    <span className="app-icon">{card.icon}</span>
                    <strong>{card.title}</strong>
                    <small>{card.copy}</small>
                  </button>
                ))}
              </section>

              <section className="creation-panel">
                <div>
                  <span className="mini-label">KidOS Create</span>
                  <h2>What do you want to create?</h2>
                  <p>KidOS opens only the tools allowed for your profile.</p>
                </div>
                <form className="creation-bar" onSubmit={createWorkspace}>
                  <label className="sr-only" htmlFor="creation-request">Ask KidOS</label>
                  <input
                    id="creation-request"
                    value={prompt}
                    onChange={(event) => setPrompt(event.target.value)}
                    placeholder="Make a space story..."
                    aria-label="Ask KidOS"
                  />
                  <button type="submit">Create</button>
                </form>
              </section>

              <section className="quote-card">
                <span>☀️</span>
                <div>
                  <h2>“The future belongs to curious minds.”</h2>
                  <p>Keep exploring, learning, and creating.</p>
                </div>
              </section>
            </>
          ) : activeSection === 'Create' ? (
            <section className="module-panel">
              <span className="module-icon">🎨</span>
              <p className="eyebrow">Create safely</p>
              <h1>What do you want to create?</h1>
              <p className="module-copy">Stories, drawings, presentations, and beginner coding start here.</p>
              <form className="creation-bar module-form" onSubmit={createWorkspace}>
                <label className="sr-only" htmlFor="creation-request-module">Ask KidOS</label>
                <input
                  id="creation-request-module"
                  value={prompt}
                  onChange={(event) => setPrompt(event.target.value)}
                  placeholder="Make a space story..."
                  aria-label="Ask KidOS"
                />
                <button type="submit">Create</button>
              </form>
            </section>
          ) : activeSection === 'Search' ? (
            <section className="module-panel">
              <span className="module-icon">🔎</span>
              <p className="eyebrow">Safe Search</p>
              <h1>Explore the web safely</h1>
              <p className="module-copy">KidOS checks destinations against the active safety policy before opening them.</p>
              <form className="protected-web-form" onSubmit={checkSite}>
                <label htmlFor="protected-web-address">Protected web address</label>
                <div>
                  <input
                    id="protected-web-address"
                    value={url}
                    onChange={(event) => setUrl(event.target.value)}
                    placeholder="https://example.com"
                  />
                  <button type="submit">Check site</button>
                </div>
              </form>
              {navigationState ? <p className="navigation-status" role="status">{navigationState}</p> : null}
            </section>
          ) : (
            <section className="module-panel">
              <span className="module-icon">
                {navItems.find((item) => item.label === activeSection)?.icon}
              </span>
              <p className="eyebrow">KidOS module</p>
              <h1>{activeSection}</h1>
              <p className="module-copy">
                {sectionDescriptions[activeSection as Exclude<Section, 'Home' | 'Create' | 'Search'>]}
              </p>
              {activeSection === 'Messages' ? (
                <div className="module-card-grid">
                  <button type="button" className="module-action-card" onClick={() => openTool('Messages')}>
                    <span>✉</span><strong>Family Messages</strong><small>Message approved family contacts inside KidOS.</small>
                  </button>
                </div>
              ) : moduleCards[activeSection]?.length ? (
                <div className="module-card-grid">
                  {moduleCards[activeSection]?.map((item) => (
                    <button type="button" className="module-action-card" key={item.title} onClick={() => openTool(item.title)}>
                      <span>{item.icon}</span>
                      <strong>{item.title}</strong>
                      <small>{item.copy}</small>
                    </button>
                  ))}
                </div>
              ) : (
                <div className="coming-soon-card">
                  <strong>Protected module</strong>
                  <span>This area is available only to approved KidOS contacts and services.</span>
                </div>
              )}
            </section>
          )}
        </div>

        <footer className="dock" aria-label="KidOS dock">
          {[
            ['📁', 'Files'],
            ['🌐', 'Browser'],
            ['📝', 'Notes'],
            ['🧮', 'Calculator'],
            ['📷', 'Camera'],
            ['🎵', 'Music'],
            ['🗑️', 'Trash'],
          ].map(([icon, label]) => (
            <button type="button" key={label} title={label} onClick={() => {
              if (label === 'Browser') openSection('Search');
              else if (label === 'Notes' || label === 'Calculator' || label === 'Music' || label === 'Files' || label === 'Camera') openTool(label);
            }}>
              <span>{icon}</span>
              <small>{label}</small>
            </button>
          ))}
        </footer>
      </section>
    </main>
  );
}
