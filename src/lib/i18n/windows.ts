import type { Locale } from '$lib/generated/i18n';

const english = {
  productKind: 'PIXEL UTILITY',
  maker: 'CODEX & XUE',
  checklists: 'CHECKLISTS',
  active: 'ACTIVE',
  done: 'DONE',
  system: 'SYSTEM',
  status: 'STATUS',
  completedTasks: 'COMPLETED TASKS',
  showCompleted: 'Show completed',
  hideCompleted: 'Hide completed',
  newListPlaceholder: 'List name',
  automaticUpdateChecks: 'Automatic update checks',
  automaticUpdateChecksDetail: 'Checks only. Download and installation always require your click.',
  currentVersion: 'Current version',
  downloading: 'Downloading',
  windowsIntegration: 'Windows integration',
  startWithWindows: 'Start with Windows',
  startWithWindowsDetail: 'Starts minimized so scheduled reminders stay refreshed.',
  enhancedAlarm: 'Enhanced XHIGH alarm',
  enhancedAlarmDetail: 'Off by default. XHIGH uses a normal Windows notification unless you enable the persistent alarm.',
  reminderQueue: 'Windows reminder queue',
  storagePrivacy: 'Storage & privacy',
  application: 'Application',
  localData: 'Local data',
  openFolder: 'Open folder',
  legacyDataFound: 'Legacy data found',
  legacyDeleteConfirm: 'Delete the legacy Roaming database? The active Local database is not affected.',
  updateReady: 'UPDATE AVAILABLE',
  notificationIssue: 'NOTIFICATION NEEDS ATTENTION',
  alarmRinging: 'XHIGH ALARM RINGING',
  openSettings: 'Open settings',
  editChecklist: 'Edit checklist',
  saveChecklist: 'Save checklist',
  cancelChecklist: 'Cancel checklist editing',
  todoCount: 'tasks',
  syncDetail: 'SYNC',
  addChecklist: 'Add checklist'
} as const;

export type WindowsMessageKey = keyof typeof english;

const translations: Record<Locale, Record<WindowsMessageKey, string>> = {
  en: english,
  'zh-Hans': {
    productKind: '像素工具', maker: 'CODEX & XUE', checklists: '待办清单', active: '进行中', done: '已完成',
    system: '系统', status: '状态', completedTasks: '已完成待办', showCompleted: '显示已完成', hideCompleted: '隐藏已完成',
    newListPlaceholder: '清单名称', automaticUpdateChecks: '自动检查更新', automaticUpdateChecksDetail: '仅自动检查；下载与安装始终需要你点击确认。',
    currentVersion: '当前版本', downloading: '正在下载', windowsIntegration: 'Windows 集成', startWithWindows: '开机启动',
    startWithWindowsDetail: '以最小化方式启动，以便持续刷新计划提醒。', enhancedAlarm: '增强 XHIGH 闹钟',
    enhancedAlarmDetail: '默认关闭。关闭时 XHIGH 与其他任务一样使用普通 Windows 通知。', reminderQueue: 'Windows 提醒队列',
    storagePrivacy: '存储与隐私', application: '应用程序', localData: '本地数据', openFolder: '打开目录', legacyDataFound: '发现旧版数据',
    legacyDeleteConfirm: '删除旧版 Roaming 数据库？当前 Local 数据库不会受到影响。', updateReady: '发现可用更新',
    notificationIssue: '通知需要处理', alarmRinging: 'XHIGH 闹钟响铃中', openSettings: '打开设置', editChecklist: '编辑清单',
    saveChecklist: '保存清单', cancelChecklist: '取消编辑清单', todoCount: '项待办', syncDetail: '同步', addChecklist: '添加清单'
  },
  ar: {
    productKind: 'أداة بكسل', maker: 'CODEX & XUE', checklists: 'قوائم المهام', active: 'نشطة', done: 'مكتملة',
    system: 'النظام', status: 'الحالة', completedTasks: 'المهام المكتملة', showCompleted: 'إظهار المكتملة', hideCompleted: 'إخفاء المكتملة',
    newListPlaceholder: 'اسم القائمة', automaticUpdateChecks: 'التحقق التلقائي من التحديثات', automaticUpdateChecksDetail: 'التحقق فقط؛ يتطلب التنزيل والتثبيت نقرة منك دائمًا.',
    currentVersion: 'الإصدار الحالي', downloading: 'جارٍ التنزيل', windowsIntegration: 'تكامل Windows', startWithWindows: 'البدء مع Windows',
    startWithWindowsDetail: 'يبدأ مصغرًا للحفاظ على تحديث التذكيرات المجدولة.', enhancedAlarm: 'منبّه XHIGH المحسّن',
    enhancedAlarmDetail: 'متوقف افتراضيًا. يستخدم XHIGH إشعار Windows عاديًا ما لم تفعّل المنبّه المستمر.', reminderQueue: 'قائمة تذكيرات Windows',
    storagePrivacy: 'التخزين والخصوصية', application: 'التطبيق', localData: 'البيانات المحلية', openFolder: 'فتح المجلد', legacyDataFound: 'عُثر على بيانات قديمة',
    legacyDeleteConfirm: 'حذف قاعدة بيانات Roaming القديمة؟ لن تتأثر قاعدة بيانات Local النشطة.', updateReady: 'يتوفر تحديث',
    notificationIssue: 'تحتاج الإشعارات إلى الانتباه', alarmRinging: 'منبّه XHIGH يرن', openSettings: 'فتح الإعدادات', editChecklist: 'تحرير القائمة',
    saveChecklist: 'حفظ القائمة', cancelChecklist: 'إلغاء تحرير القائمة', todoCount: 'مهام', syncDetail: 'المزامنة', addChecklist: 'إضافة قائمة'
  },
  fr: {
    productKind: 'OUTIL PIXEL', maker: 'CODEX & XUE', checklists: 'LISTES', active: 'ACTIVES', done: 'TERMINÉES',
    system: 'SYSTÈME', status: 'ÉTAT', completedTasks: 'TÂCHES TERMINÉES', showCompleted: 'Afficher les terminées', hideCompleted: 'Masquer les terminées',
    newListPlaceholder: 'Nom de la liste', automaticUpdateChecks: 'Recherche automatique des mises à jour', automaticUpdateChecksDetail: 'Vérification uniquement. Le téléchargement et l’installation nécessitent toujours un clic.',
    currentVersion: 'Version actuelle', downloading: 'Téléchargement', windowsIntegration: 'Intégration Windows', startWithWindows: 'Démarrer avec Windows',
    startWithWindowsDetail: 'Démarre réduit pour maintenir les rappels planifiés à jour.', enhancedAlarm: 'Alarme XHIGH renforcée',
    enhancedAlarmDetail: 'Désactivée par défaut. XHIGH utilise une notification Windows normale sauf si vous activez l’alarme persistante.', reminderQueue: 'File des rappels Windows',
    storagePrivacy: 'Stockage et confidentialité', application: 'Application', localData: 'Données locales', openFolder: 'Ouvrir le dossier', legacyDataFound: 'Anciennes données trouvées',
    legacyDeleteConfirm: 'Supprimer l’ancienne base Roaming ? La base Local active ne sera pas affectée.', updateReady: 'MISE À JOUR DISPONIBLE',
    notificationIssue: 'NOTIFICATION À VÉRIFIER', alarmRinging: 'ALARME XHIGH EN COURS', openSettings: 'Ouvrir les réglages', editChecklist: 'Modifier la liste',
    saveChecklist: 'Enregistrer la liste', cancelChecklist: 'Annuler la modification', todoCount: 'tâches', syncDetail: 'SYNCHRO', addChecklist: 'Ajouter une liste'
  },
  ru: {
    productKind: 'ПИКСЕЛЬНЫЙ ИНСТРУМЕНТ', maker: 'CODEX & XUE', checklists: 'СПИСКИ', active: 'АКТИВНЫЕ', done: 'ГОТОВО',
    system: 'СИСТЕМА', status: 'СОСТОЯНИЕ', completedTasks: 'ВЫПОЛНЕННЫЕ ЗАДАЧИ', showCompleted: 'Показать выполненные', hideCompleted: 'Скрыть выполненные',
    newListPlaceholder: 'Название списка', automaticUpdateChecks: 'Автоматическая проверка обновлений', automaticUpdateChecksDetail: 'Только проверка. Скачивание и установка всегда требуют вашего нажатия.',
    currentVersion: 'Текущая версия', downloading: 'Загрузка', windowsIntegration: 'Интеграция Windows', startWithWindows: 'Запускать с Windows',
    startWithWindowsDetail: 'Запускается свёрнутым, чтобы обновлять запланированные напоминания.', enhancedAlarm: 'Усиленный будильник XHIGH',
    enhancedAlarmDetail: 'По умолчанию выключен. XHIGH использует обычное уведомление Windows, пока не включён постоянный будильник.', reminderQueue: 'Очередь напоминаний Windows',
    storagePrivacy: 'Хранилище и конфиденциальность', application: 'Приложение', localData: 'Локальные данные', openFolder: 'Открыть папку', legacyDataFound: 'Найдены старые данные',
    legacyDeleteConfirm: 'Удалить старую базу Roaming? Активная база Local не изменится.', updateReady: 'ДОСТУПНО ОБНОВЛЕНИЕ',
    notificationIssue: 'ТРЕБУЕТСЯ НАСТРОЙКА УВЕДОМЛЕНИЙ', alarmRinging: 'ЗВОНИТ БУДИЛЬНИК XHIGH', openSettings: 'Открыть настройки', editChecklist: 'Изменить список',
    saveChecklist: 'Сохранить список', cancelChecklist: 'Отменить изменение', todoCount: 'задач', syncDetail: 'СИНХР.', addChecklist: 'Добавить список'
  },
  es: {
    productKind: 'UTILIDAD PÍXEL', maker: 'CODEX & XUE', checklists: 'LISTAS', active: 'ACTIVAS', done: 'HECHAS',
    system: 'SISTEMA', status: 'ESTADO', completedTasks: 'TAREAS HECHAS', showCompleted: 'Mostrar hechas', hideCompleted: 'Ocultar hechas',
    newListPlaceholder: 'Nombre de la lista', automaticUpdateChecks: 'Comprobación automática de actualizaciones', automaticUpdateChecksDetail: 'Solo comprueba. La descarga y la instalación siempre requieren tu clic.',
    currentVersion: 'Versión actual', downloading: 'Descargando', windowsIntegration: 'Integración con Windows', startWithWindows: 'Iniciar con Windows',
    startWithWindowsDetail: 'Se inicia minimizada para mantener al día los recordatorios programados.', enhancedAlarm: 'Alarma XHIGH mejorada',
    enhancedAlarmDetail: 'Desactivada de forma predeterminada. XHIGH usa una notificación normal de Windows salvo que actives la alarma persistente.', reminderQueue: 'Cola de recordatorios de Windows',
    storagePrivacy: 'Almacenamiento y privacidad', application: 'Aplicación', localData: 'Datos locales', openFolder: 'Abrir carpeta', legacyDataFound: 'Se encontraron datos antiguos',
    legacyDeleteConfirm: '¿Eliminar la base Roaming antigua? La base Local activa no se verá afectada.', updateReady: 'ACTUALIZACIÓN DISPONIBLE',
    notificationIssue: 'NOTIFICACIÓN REQUIERE ATENCIÓN', alarmRinging: 'ALARMA XHIGH SONANDO', openSettings: 'Abrir ajustes', editChecklist: 'Editar lista',
    saveChecklist: 'Guardar lista', cancelChecklist: 'Cancelar edición', todoCount: 'tareas', syncDetail: 'SINCRONIZAR', addChecklist: 'Añadir lista'
  }
};

export function windowsMessage(locale: Locale, key: WindowsMessageKey): string {
  return translations[locale][key];
}
