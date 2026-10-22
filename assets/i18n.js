// Presentation-only localization. Source text is retained for reversible language changes.
let language = 'ru';
const translations = {
 'Фильтр процессов':'Process filter',
 'Все процессы':'All processes',
 'Ограниченная привязка':'Restricted affinity',
 'Порог активности':'Activity threshold',
 'Любая активность':'Any activity',
 'Сбросить':'Reset',
 'Для активности запустите CoreScope от администратора':'Run CoreScope as administrator to track activity',
 'Активность для нескольких групп CPU пока недоступна':'Activity across multiple CPU groups is not supported yet',
 'Выполнение за 1 секунду · задержка ':'Execution over 1 second · delay ',
 'Выполнение за 1 секунду · ETW':'Execution over 1 second · ETW',
 'Процесс завершён или недоступен':'Process exited or unavailable',
 'Ожидание событий ETW…':'Waiting for ETW events…',
 'События ETW задерживаются':'ETW events are delayed',
 'Сбор ETW остановлен':'ETW collection stopped',
 'Не удалось запустить ETW':'Could not start ETW',
 'Нераспознанное время:':'Unattributed time:',
 'Потеряно событий ETW:':'ETW events lost:',
 'Недостаточно данных':'Insufficient data',
 'Доля времени ядра':'Share of CPU time',
 'Активность':'Activity',
 'Простой':'Idle',
 'ACPI и накопители читаются через Windows. Расширенные датчики работают внутри CoreScope.':'ACPI and storage are read through Windows. Extended sensors run inside CoreScope.',
 'ACPI — термозона, не отдельное ядро.':'ACPI is a thermal zone, not an individual CPU core.',
 'Проверяем Windows и встроенный модуль.':'Checking Windows and the integrated module.',
 'Встроенный модуль включён':'Integrated module enabled',
 'Встроенный модуль выключен':'Integrated module disabled',
 'Температура CPU недоступна':'CPU temperature unavailable',
 'Диагностика датчиков':'Sensor diagnostics',
 'Расширенные датчики':'Extended sensors',
 'Включить расширенные датчики':'Enable extended sensors',
 'Привязка для нескольких групп CPU пока недоступна':'Affinity across multiple CPU groups is not supported yet',
 'Процесс уже завершён или PID был переиспользован. Обновите выбор.':'Process exited or PID was reused. Select it again.',
 'Процесс завершён или PID изменился':'Process exited or PID changed',
 'Процесс изменился. Выберите его заново.':'Process changed. Select it again.',
 'Нельзя запретить последнее разрешённое ядро':'Cannot deny the last allowed CPU',
 'Выберите хотя бы одно ядро':'Select at least one CPU',
 'Недопустимый номер CPU':'Invalid CPU number',
 'Нужен полный путь к .exe':'Enter the full path to an .exe',
 'Запуск без аргументов. Приложения, передающие запуск другому процессу, могут не сохранить привязку.':'No arguments. Apps that delegate launch to another process may not retain affinity.',
 'Запустить программу на выбранных CPU':'Launch an app on selected CPUs',
 'Изометрическая карта CPU и оперативной памяти':'Isometric CPU and RAM map',
 'Процесс завершён. Выберите другой процесс.':'Process exited. Select another process.',
 'Температурные датчики · максимум':'Temperature sensors · maximum',
 'Процессы / логические CPU':'Processes / logical CPUs',
 'Разрешить исполнение на CPU':'Allow execution on CPUs',
 'Последнее разрешённое ядро':'Last allowed CPU',
 'Обновление каждую секунду':'Updates every second',
 'Обновить карту памяти':'Refresh memory map',
 'Датчики пока недоступны':'Sensors unavailable',
 'нажмите для фильтра процессов':'click to filter processes',
 'Связь потеряна · повторяем…':'Disconnected · retrying…',
 'Доступ к ядру по affinity':'CPU access by affinity',
 'Привязка применена: CPU':'Affinity applied: CPU',
 'Запущен процесс PID':'Started process PID',
 'Закрывать окно в трей':'Close window to tray',
 'История температур':'Temperature history',
 'Сохранить настройки':'Save settings',
 'Настройки сохранены':'Settings saved',
 'Запустить датчики':'Start sensors',
 'Ожидание новых замеров':'Waiting for samples',
 'Загрузка процессора':'CPU utilization',
 'Физическая память':'Physical memory',
 'Выберите ядро на карте.':'Select a CPU on the map.',
 'Процессы не найдены':'No processes found',
 'Привязка недоступна.':'Affinity unavailable.',
 'разрешённых CPU':'allowed CPUs',
 'Разрешённые CPU':'Allowed CPUs',
 'Виртуальная память':'Virtual memory',
 'Процессы ядра':'CPU processes',
 'Выберите процесс.':'Select a process.',
 'Рабочий набор':'Working set',
 'Полный путь к exe':'Full executable path',
 'логических CPU':'logical CPUs',
 'Данные устарели':'Stale data',
 'Открыть процесс':'Open process',
 'Процесс или PID':'Process or PID',
 'Поиск процессов':'Search processes',
 'Поиск на ядре':'Search on CPU',
 'Имя или PID…':'Name or PID…',
 'ожидание датчиков':'waiting for sensors',
 'от администратора.':'as administrator.',
 'committed-областей':'committed regions',
 'частичная карта':'partial map',
 'Читаем карту…':'Reading map…',
 'Применяем…':'Applying…',
 'Подключение…':'Connecting…',
 'Первый замер…':'First sample…',
 'Запустить на CPU':'Launch on CPUs',
 'Выберите ядро':'Select a CPU',
 'Карта системы':'System map',
 'Этот компьютер':'This computer',
 'Компьютер':'Computer',
 'Загрузка…':'Loading…',
 'нет датчиков':'no sensors',
 'Нет доступа':'Access denied',
 'Единственное':'Only CPU',
 'Разрешено':'Allowed',
 'Запрещено':'Denied',
 'Разрешён':'Allowed',
 'Запрещён':'Denied',
 'Запретить':'Deny',
 'Разрешить':'Allow',
 'Применить':'Apply',
 'Все CPU':'All CPUs',
 'Запустите':'Run',
 'Запустить':'Launch',
 'Настройки':'Settings',
 'Отмена':'Cancel',
 'Процессы':'Processes',
 'Процесс':'Process',
 'ДАТЧИКОВ':'SENSORS',
 'ДАТЧИКИ':'SENSORS',
 'Датчики':'Sensors',
 'ПРОЦЕССОР':'PROCESSOR',
 'ПАМЯТЬ':'MEMORY',
 'Фильтр CPU':'Filter CPU',
 'загрузка':'load',
 'занято':'used',
 'В эфире':'Live',
 'Ошибка':'Error',
 'из ':'of ',
};
const pairs = Object.entries(translations).sort((a,b)=>b[0].length-a[0].length);
const originals = new WeakMap();
function translateText(text) {
 if(language==='ru') return text;
 let normalized=text.replace(/\s+/g,' ').trim();
 if(translations[normalized]) return text.replace(text.trim(),translations[normalized]);
 // A single pass prevents translating pieces of another replacement.
 return text.replace(new RegExp(pairs.map(([s])=>s.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')).join('|'),'g'),s=>translations[s]);
}
function localize() {
 observer.disconnect();
 const walk=document.createTreeWalker(document.body,NodeFilter.SHOW_TEXT);
 let n;
 while(n=walk.nextNode()) {
  if(n.parentElement?.closest('script,style,.selected-title,.processlink strong,tr[data-pid] td:first-child')) continue;
  const previous=originals.get(n);
  const source=previous && n.nodeValue===previous.output ? previous.source : n.nodeValue;
  const output=translateText(source);
  if(n.nodeValue!==output) n.nodeValue=output;
  originals.set(n,{source,output});
 }
 for(const el of document.querySelectorAll('[placeholder],[title],[aria-label]')) {
  let saved=originals.get(el)||{};
  for(const attr of ['placeholder','title','aria-label']) { if(!el.hasAttribute(attr))continue; const current=el.getAttribute(attr), old=saved[attr]; const source=old&&current===old.output?old.source:current; const output=translateText(source); if(output!==current)el.setAttribute(attr,output); saved[attr]={source,output}; }
  originals.set(el,saved);
 }
 document.documentElement.lang=language;
 observer.observe(document.body,{subtree:true,childList:true,characterData:true,attributes:true,attributeFilter:['placeholder','title','aria-label']});
}
const observer=new MutationObserver(localize);
function setLanguage(value) { language=value==='en'?'en':'ru'; document.getElementById('language').value=language; localize(); }
localize();
