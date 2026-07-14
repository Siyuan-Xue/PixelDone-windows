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
  signInToSyncAndroid: 'Sign in to sync with Android.',
  addChecklist: 'Add checklist',
  changePassword: 'Change password',
  currentPassword: 'Current password',
  newPassword: 'New password',
  confirmPassword: 'Confirm new password',
  changingPassword: 'Changing password…',
  thisDevice: 'This device', cloudVersion: 'Cloud version', useThisDevice: 'Use this device', useCloudVersion: 'Use cloud version',
  conflictPosition: 'Position {value}', statusActive: 'Active', statusCompleted: 'Completed', emptyValue: 'None'
} as const;

export type WindowsMessageKey = keyof typeof english;

export type WindowsAuthValidationKey = 'invalidEmail' | 'passwordTooShort';

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
    saveChecklist: '保存清单', cancelChecklist: '取消编辑清单', todoCount: '项待办', syncDetail: '同步',
    signInToSyncAndroid: '登录后可与 Android 同步。', addChecklist: '添加清单',
    changePassword: '修改密码', currentPassword: '当前密码', newPassword: '新密码', confirmPassword: '确认新密码', changingPassword: '正在修改密码…',
    thisDevice: '此设备', cloudVersion: '云端版本', useThisDevice: '使用此设备版本', useCloudVersion: '使用云端版本',
    conflictPosition: '清单位置 {value}', statusActive: '未完成', statusCompleted: '已完成', emptyValue: '无'
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
    saveChecklist: 'حفظ القائمة', cancelChecklist: 'إلغاء تحرير القائمة', todoCount: 'مهام', syncDetail: 'المزامنة',
    signInToSyncAndroid: 'سجّل الدخول للمزامنة مع Android.', addChecklist: 'إضافة قائمة',
    changePassword: 'تغيير كلمة المرور', currentPassword: 'كلمة المرور الحالية', newPassword: 'كلمة المرور الجديدة', confirmPassword: 'تأكيد كلمة المرور الجديدة', changingPassword: 'جارٍ تغيير كلمة المرور…',
    thisDevice: 'هذا الجهاز', cloudVersion: 'إصدار السحابة', useThisDevice: 'استخدام إصدار هذا الجهاز', useCloudVersion: 'استخدام إصدار السحابة',
    conflictPosition: 'الموضع {value}', statusActive: 'نشطة', statusCompleted: 'مكتملة', emptyValue: 'لا شيء'
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
    saveChecklist: 'Enregistrer la liste', cancelChecklist: 'Annuler la modification', todoCount: 'tâches', syncDetail: 'SYNCHRO',
    signInToSyncAndroid: 'Connectez-vous pour synchroniser avec Android.', addChecklist: 'Ajouter une liste',
    changePassword: 'Modifier le mot de passe', currentPassword: 'Mot de passe actuel', newPassword: 'Nouveau mot de passe', confirmPassword: 'Confirmer le nouveau mot de passe', changingPassword: 'Modification du mot de passe…',
    thisDevice: 'Cet appareil', cloudVersion: 'Version cloud', useThisDevice: 'Utiliser cet appareil', useCloudVersion: 'Utiliser la version cloud',
    conflictPosition: 'Position {value}', statusActive: 'Active', statusCompleted: 'Terminée', emptyValue: 'Aucun'
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
    saveChecklist: 'Сохранить список', cancelChecklist: 'Отменить изменение', todoCount: 'задач', syncDetail: 'СИНХР.',
    signInToSyncAndroid: 'Войдите, чтобы синхронизироваться с Android.', addChecklist: 'Добавить список',
    changePassword: 'Изменить пароль', currentPassword: 'Текущий пароль', newPassword: 'Новый пароль', confirmPassword: 'Подтвердите новый пароль', changingPassword: 'Изменение пароля…',
    thisDevice: 'Это устройство', cloudVersion: 'Облачная версия', useThisDevice: 'Использовать версию устройства', useCloudVersion: 'Использовать облачную версию',
    conflictPosition: 'Позиция {value}', statusActive: 'Активная', statusCompleted: 'Выполнена', emptyValue: 'Нет'
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
    saveChecklist: 'Guardar lista', cancelChecklist: 'Cancelar edición', todoCount: 'tareas', syncDetail: 'SINCRONIZAR',
    signInToSyncAndroid: 'Inicia sesión para sincronizar con Android.', addChecklist: 'Añadir lista',
    changePassword: 'Cambiar contraseña', currentPassword: 'Contraseña actual', newPassword: 'Nueva contraseña', confirmPassword: 'Confirmar nueva contraseña', changingPassword: 'Cambiando contraseña…',
    thisDevice: 'Este dispositivo', cloudVersion: 'Versión en la nube', useThisDevice: 'Usar este dispositivo', useCloudVersion: 'Usar la versión en la nube',
    conflictPosition: 'Posición {value}', statusActive: 'Activa', statusCompleted: 'Completada', emptyValue: 'Ninguno'
  }
};

const authValidationTranslations: Record<Locale, Record<WindowsAuthValidationKey, string>> = {
  en: {
    invalidEmail: 'Enter a valid email address.',
    passwordTooShort: 'Password must contain at least 6 characters.'
  },
  'zh-Hans': {
    invalidEmail: '\u8bf7\u8f93\u5165\u6709\u6548\u7684\u90ae\u7bb1\u5730\u5740\u3002',
    passwordTooShort: '\u5bc6\u7801\u81f3\u5c11\u9700\u8981 6 \u4e2a\u5b57\u7b26\u3002'
  },
  ar: {
    invalidEmail: '\u0623\u062f\u062e\u0644 \u0639\u0646\u0648\u0627\u0646 \u0628\u0631\u064a\u062f \u0625\u0644\u0643\u062a\u0631\u0648\u0646\u064a \u0635\u0627\u0644\u062d\u064b\u0627.',
    passwordTooShort: '\u064a\u062c\u0628 \u0623\u0646 \u062a\u062a\u0643\u0648\u0646 \u0643\u0644\u0645\u0629 \u0627\u0644\u0645\u0631\u0648\u0631 \u0645\u0646 6 \u0623\u062d\u0631\u0641 \u0639\u0644\u0649 \u0627\u0644\u0623\u0642\u0644.'
  },
  fr: {
    invalidEmail: 'Saisissez une adresse e-mail valide.',
    passwordTooShort: 'Le mot de passe doit contenir au moins 6 caract\u00e8res.'
  },
  ru: {
    invalidEmail: '\u0412\u0432\u0435\u0434\u0438\u0442\u0435 \u043a\u043e\u0440\u0440\u0435\u043a\u0442\u043d\u044b\u0439 \u0430\u0434\u0440\u0435\u0441 \u044d\u043b\u0435\u043a\u0442\u0440\u043e\u043d\u043d\u043e\u0439 \u043f\u043e\u0447\u0442\u044b.',
    passwordTooShort: '\u041f\u0430\u0440\u043e\u043b\u044c \u0434\u043e\u043b\u0436\u0435\u043d \u0441\u043e\u0434\u0435\u0440\u0436\u0430\u0442\u044c \u043d\u0435 \u043c\u0435\u043d\u0435\u0435 6 \u0441\u0438\u043c\u0432\u043e\u043b\u043e\u0432.'
  },
  es: {
    invalidEmail: 'Introduce una direcci\u00f3n de correo v\u00e1lida.',
    passwordTooShort: 'La contrase\u00f1a debe tener al menos 6 caracteres.'
  }
};

export function windowsMessage(locale: Locale, key: WindowsMessageKey): string {
  return translations[locale][key];
}

export function windowsAuthValidationMessage(
  locale: Locale,
  key: WindowsAuthValidationKey
): string {
  return authValidationTranslations[locale][key];
}
