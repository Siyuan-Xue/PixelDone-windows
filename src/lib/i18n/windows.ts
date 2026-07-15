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

export type WindowsReliabilityMessageKey =
  | 'passwordFieldsRequired'
  | 'passwordTooShort'
  | 'passwordMustDiffer'
  | 'passwordsDoNotMatch'
  | 'passwordChanged'
  | 'staleRevisionAgain'
  | 'targetChanged'
  | 'syncNetworkRetrying'
  | 'syncAuthExpired'
  | 'syncServerUpdateRequired'
  | 'syncLocalStorageError'
  | 'syncRemoteDataInvalid'
  | 'syncUnknown';

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

const reliabilityTranslations: Record<Locale, Record<WindowsReliabilityMessageKey, string>> = {
  en: {
    passwordFieldsRequired: 'All password fields are required.',
    passwordTooShort: 'The new password must contain at least 6 characters.',
    passwordMustDiffer: 'Choose a new password that differs from the current password.',
    passwordsDoNotMatch: 'The new passwords do not match.',
    passwordChanged: 'Password changed. Sign in again.',
    staleRevisionAgain: 'The data changed again. Please perform the action again.',
    targetChanged: 'Another device changed or deleted this item. The latest data has been loaded.',
    syncNetworkRetrying: 'The connection was interrupted. PixelDone is retrying automatically.',
    syncAuthExpired: 'Your sign-in expired. Sign in again to resume sync.',
    syncServerUpdateRequired: 'The cloud version is incompatible. Update PixelDone and try again.',
    syncLocalStorageError: 'Sync data could not be saved. Check local storage and try again.',
    syncRemoteDataInvalid: 'The cloud returned unrecognized data. Sync has been paused.',
    syncUnknown: 'Sync encountered an unknown error. Try again.'
  },
  'zh-Hans': {
    passwordFieldsRequired: '\u6240\u6709\u5bc6\u7801\u5b57\u6bb5\u5747\u4e3a\u5fc5\u586b\u9879\u3002',
    passwordTooShort: '\u65b0\u5bc6\u7801\u81f3\u5c11\u9700\u8981 6 \u4e2a\u5b57\u7b26\u3002',
    passwordMustDiffer: '\u8bf7\u9009\u62e9\u4e0e\u5f53\u524d\u5bc6\u7801\u4e0d\u540c\u7684\u65b0\u5bc6\u7801\u3002',
    passwordsDoNotMatch: '\u4e24\u6b21\u8f93\u5165\u7684\u65b0\u5bc6\u7801\u4e0d\u4e00\u81f4\u3002',
    passwordChanged: '\u5bc6\u7801\u5df2\u4fee\u6539\u3002\u8bf7\u91cd\u65b0\u767b\u5f55\u3002',
    staleRevisionAgain: '\u6570\u636e\u518d\u6b21\u53d1\u751f\u53d8\u5316\uff0c\u8bf7\u91cd\u65b0\u6267\u884c\u64cd\u4f5c\u3002',
    targetChanged: '\u8be5\u5185\u5bb9\u5df2\u88ab\u5176\u4ed6\u8bbe\u5907\u4fee\u6539\u6216\u5220\u9664\uff0c\u5df2\u52a0\u8f7d\u6700\u65b0\u6570\u636e\u3002',
    syncNetworkRetrying: '\u7f51\u7edc\u8fde\u63a5\u4e2d\u65ad\u3002PixelDone \u6b63\u5728\u81ea\u52a8\u91cd\u8bd5\u3002',
    syncAuthExpired: '\u767b\u5f55\u5df2\u8fc7\u671f\uff0c\u8bf7\u91cd\u65b0\u767b\u5f55\u4ee5\u7ee7\u7eed\u540c\u6b65\u3002',
    syncServerUpdateRequired: '\u4e91\u7aef\u7248\u672c\u4e0d\u517c\u5bb9\uff0c\u8bf7\u66f4\u65b0 PixelDone \u540e\u91cd\u8bd5\u3002',
    syncLocalStorageError: '\u65e0\u6cd5\u4fdd\u5b58\u540c\u6b65\u6570\u636e\uff0c\u8bf7\u68c0\u67e5\u672c\u5730\u5b58\u50a8\u540e\u91cd\u8bd5\u3002',
    syncRemoteDataInvalid: '\u4e91\u7aef\u8fd4\u56de\u4e86\u65e0\u6cd5\u8bc6\u522b\u7684\u6570\u636e\uff1b\u540c\u6b65\u5df2\u6682\u505c\u3002',
    syncUnknown: '\u540c\u6b65\u9047\u5230\u672a\u77e5\u9519\u8bef\uff0c\u8bf7\u91cd\u8bd5\u3002'
  },
  ar: {
    passwordFieldsRequired: '\u062c\u0645\u064a\u0639 \u062d\u0642\u0648\u0644 \u0643\u0644\u0645\u0629 \u0627\u0644\u0645\u0631\u0648\u0631 \u0645\u0637\u0644\u0648\u0628\u0629.',
    passwordTooShort: '\u064a\u062c\u0628 \u0623\u0646 \u062a\u062a\u0643\u0648\u0646 \u0643\u0644\u0645\u0629 \u0627\u0644\u0645\u0631\u0648\u0631 \u0627\u0644\u062c\u062f\u064a\u062f\u0629 \u0645\u0646 6 \u0623\u062d\u0631\u0641 \u0639\u0644\u0649 \u0627\u0644\u0623\u0642\u0644.',
    passwordMustDiffer: '\u0627\u062e\u062a\u0631 \u0643\u0644\u0645\u0629 \u0645\u0631\u0648\u0631 \u062c\u062f\u064a\u062f\u0629 \u0645\u062e\u062a\u0644\u0641\u0629 \u0639\u0646 \u0627\u0644\u062d\u0627\u0644\u064a\u0629.',
    passwordsDoNotMatch: '\u0643\u0644\u0645\u062a\u0627 \u0627\u0644\u0645\u0631\u0648\u0631 \u0627\u0644\u062c\u062f\u064a\u062f\u062a\u0627\u0646 \u063a\u064a\u0631 \u0645\u062a\u0637\u0627\u0628\u0642\u062a\u064a\u0646.',
    passwordChanged: '\u062a\u0645 \u062a\u063a\u064a\u064a\u0631 \u0643\u0644\u0645\u0629 \u0627\u0644\u0645\u0631\u0648\u0631. \u0633\u062c\u0651\u0644 \u0627\u0644\u062f\u062e\u0648\u0644 \u0645\u0631\u0629 \u0623\u062e\u0631\u0649.',
    staleRevisionAgain: '\u062a\u063a\u064a\u0651\u0631\u062a \u0627\u0644\u0628\u064a\u0627\u0646\u0627\u062a \u0645\u0631\u0629 \u0623\u062e\u0631\u0649. \u0623\u0639\u062f \u062a\u0646\u0641\u064a\u0630 \u0627\u0644\u0625\u062c\u0631\u0627\u0621.',
    targetChanged: '\u0639\u062f\u0651\u0644 \u062c\u0647\u0627\u0632 \u0622\u062e\u0631 \u0647\u0630\u0627 \u0627\u0644\u0639\u0646\u0635\u0631 \u0623\u0648 \u062d\u0630\u0641\u0647. \u062a\u0645 \u062a\u062d\u0645\u064a\u0644 \u0623\u062d\u062f\u062b \u0627\u0644\u0628\u064a\u0627\u0646\u0627\u062a.',
    syncNetworkRetrying: '\u0627\u0646\u0642\u0637\u0639 \u0627\u0644\u0627\u062a\u0635\u0627\u0644. \u064a\u062d\u0627\u0648\u0644 PixelDone \u0627\u0644\u0645\u0632\u0627\u0645\u0646\u0629 \u062a\u0644\u0642\u0627\u0626\u064a\u064b\u0627.',
    syncAuthExpired: '\u0627\u0646\u062a\u0647\u062a \u0635\u0644\u0627\u062d\u064a\u0629 \u062a\u0633\u062c\u064a\u0644 \u0627\u0644\u062f\u062e\u0648\u0644. \u0633\u062c\u0651\u0644 \u0627\u0644\u062f\u062e\u0648\u0644 \u0645\u0631\u0629 \u0623\u062e\u0631\u0649.',
    syncServerUpdateRequired: '\u0625\u0635\u062f\u0627\u0631 \u0627\u0644\u062e\u0627\u062f\u0645 \u063a\u064a\u0631 \u0645\u062a\u0648\u0627\u0641\u0642. \u062d\u062f\u0651\u062b PixelDone \u062b\u0645 \u0623\u0639\u062f \u0627\u0644\u0645\u062d\u0627\u0648\u0644\u0629.',
    syncLocalStorageError: '\u062a\u0639\u0630\u0631 \u062d\u0641\u0638 \u0628\u064a\u0627\u0646\u0627\u062a \u0627\u0644\u0645\u0632\u0627\u0645\u0646\u0629 \u0645\u062d\u0644\u064a\u064b\u0627. \u062a\u062d\u0642\u0642 \u0645\u0646 \u0627\u0644\u062a\u062e\u0632\u064a\u0646 \u0648\u062d\u0627\u0648\u0644 \u0645\u0631\u0629 \u0623\u062e\u0631\u0649.',
    syncRemoteDataInvalid: '\u0623\u0639\u0627\u062f \u0627\u0644\u062e\u0627\u062f\u0645 \u0628\u064a\u0627\u0646\u0627\u062a \u063a\u064a\u0631 \u0645\u0639\u0631\u0648\u0641\u0629. \u062a\u0645 \u0625\u064a\u0642\u0627\u0641 \u0627\u0644\u0645\u0632\u0627\u0645\u0646\u0629.',
    syncUnknown: '\u062d\u062f\u062b \u062e\u0637\u0623 \u0645\u0632\u0627\u0645\u0646\u0629 \u063a\u064a\u0631 \u0645\u0639\u0631\u0648\u0641. \u062d\u0627\u0648\u0644 \u0645\u0631\u0629 \u0623\u062e\u0631\u0649.'
  },
  fr: {
    passwordFieldsRequired: 'Tous les champs de mot de passe sont obligatoires.',
    passwordTooShort: 'Le nouveau mot de passe doit contenir au moins 6 caract\u00e8res.',
    passwordMustDiffer: 'Choisissez un nouveau mot de passe diff\u00e9rent de l\u2019actuel.',
    passwordsDoNotMatch: 'Les nouveaux mots de passe ne correspondent pas.',
    passwordChanged: 'Mot de passe modifi\u00e9. Reconnectez-vous.',
    staleRevisionAgain: 'Les donn\u00e9es ont encore chang\u00e9. Recommencez l\u2019action.',
    targetChanged: 'Un autre appareil a modifi\u00e9 ou supprim\u00e9 cet \u00e9l\u00e9ment. Les donn\u00e9es r\u00e9centes sont charg\u00e9es.',
    syncNetworkRetrying: 'La connexion a \u00e9t\u00e9 interrompue. PixelDone r\u00e9essaie automatiquement.',
    syncAuthExpired: 'Votre connexion a expir\u00e9. Reconnectez-vous pour reprendre la synchronisation.',
    syncServerUpdateRequired: 'La version cloud est incompatible. Mettez PixelDone \u00e0 jour puis r\u00e9essayez.',
    syncLocalStorageError: 'Les donn\u00e9es de synchronisation n\u2019ont pas pu \u00eatre enregistr\u00e9es. V\u00e9rifiez le stockage local.',
    syncRemoteDataInvalid: 'Le cloud a renvoy\u00e9 des donn\u00e9es inconnues. La synchronisation est suspendue.',
    syncUnknown: 'Une erreur de synchronisation inconnue est survenue. R\u00e9essayez.'
  },
  ru: {
    passwordFieldsRequired: '\u0412\u0441\u0435 \u043f\u043e\u043b\u044f \u043f\u0430\u0440\u043e\u043b\u044f \u043e\u0431\u044f\u0437\u0430\u0442\u0435\u043b\u044c\u043d\u044b.',
    passwordTooShort: '\u041d\u043e\u0432\u044b\u0439 \u043f\u0430\u0440\u043e\u043b\u044c \u0434\u043e\u043b\u0436\u0435\u043d \u0441\u043e\u0434\u0435\u0440\u0436\u0430\u0442\u044c \u043d\u0435 \u043c\u0435\u043d\u0435\u0435 6 \u0441\u0438\u043c\u0432\u043e\u043b\u043e\u0432.',
    passwordMustDiffer: '\u0412\u044b\u0431\u0435\u0440\u0438\u0442\u0435 \u043d\u043e\u0432\u044b\u0439 \u043f\u0430\u0440\u043e\u043b\u044c, \u043e\u0442\u043b\u0438\u0447\u0430\u044e\u0449\u0438\u0439\u0441\u044f \u043e\u0442 \u0442\u0435\u043a\u0443\u0449\u0435\u0433\u043e.',
    passwordsDoNotMatch: '\u041d\u043e\u0432\u044b\u0435 \u043f\u0430\u0440\u043e\u043b\u0438 \u043d\u0435 \u0441\u043e\u0432\u043f\u0430\u0434\u0430\u044e\u0442.',
    passwordChanged: '\u041f\u0430\u0440\u043e\u043b\u044c \u0438\u0437\u043c\u0435\u043d\u0451\u043d. \u0412\u043e\u0439\u0434\u0438\u0442\u0435 \u0441\u043d\u043e\u0432\u0430.',
    staleRevisionAgain: '\u0414\u0430\u043d\u043d\u044b\u0435 \u0441\u043d\u043e\u0432\u0430 \u0438\u0437\u043c\u0435\u043d\u0438\u043b\u0438\u0441\u044c. \u041f\u043e\u0432\u0442\u043e\u0440\u0438\u0442\u0435 \u0434\u0435\u0439\u0441\u0442\u0432\u0438\u0435.',
    targetChanged: '\u0414\u0440\u0443\u0433\u043e\u0435 \u0443\u0441\u0442\u0440\u043e\u0439\u0441\u0442\u0432\u043e \u0438\u0437\u043c\u0435\u043d\u0438\u043b\u043e \u0438\u043b\u0438 \u0443\u0434\u0430\u043b\u0438\u043b\u043e \u044d\u0442\u043e\u0442 \u044d\u043b\u0435\u043c\u0435\u043d\u0442. \u0417\u0430\u0433\u0440\u0443\u0436\u0435\u043d\u044b \u0430\u043a\u0442\u0443\u0430\u043b\u044c\u043d\u044b\u0435 \u0434\u0430\u043d\u043d\u044b\u0435.',
    syncNetworkRetrying: '\u0421\u043e\u0435\u0434\u0438\u043d\u0435\u043d\u0438\u0435 \u043f\u0440\u0435\u0440\u0432\u0430\u043d\u043e. PixelDone \u043f\u043e\u0432\u0442\u043e\u0440\u044f\u0435\u0442 \u043f\u043e\u043f\u044b\u0442\u043a\u0443 \u0430\u0432\u0442\u043e\u043c\u0430\u0442\u0438\u0447\u0435\u0441\u043a\u0438.',
    syncAuthExpired: '\u0421\u0435\u0430\u043d\u0441 \u0432\u0445\u043e\u0434\u0430 \u0438\u0441\u0442\u0451\u043a. \u0412\u043e\u0439\u0434\u0438\u0442\u0435 \u0441\u043d\u043e\u0432\u0430.',
    syncServerUpdateRequired: '\u0412\u0435\u0440\u0441\u0438\u044f \u043e\u0431\u043b\u0430\u043a\u0430 \u043d\u0435\u0441\u043e\u0432\u043c\u0435\u0441\u0442\u0438\u043c\u0430. \u041e\u0431\u043d\u043e\u0432\u0438\u0442\u0435 PixelDone \u0438 \u043f\u043e\u0432\u0442\u043e\u0440\u0438\u0442\u0435.',
    syncLocalStorageError: '\u041d\u0435 \u0443\u0434\u0430\u043b\u043e\u0441\u044c \u0441\u043e\u0445\u0440\u0430\u043d\u0438\u0442\u044c \u0434\u0430\u043d\u043d\u044b\u0435 \u0441\u0438\u043d\u0445\u0440\u043e\u043d\u0438\u0437\u0430\u0446\u0438\u0438. \u041f\u0440\u043e\u0432\u0435\u0440\u044c\u0442\u0435 \u043b\u043e\u043a\u0430\u043b\u044c\u043d\u043e\u0435 \u0445\u0440\u0430\u043d\u0438\u043b\u0438\u0449\u0435.',
    syncRemoteDataInvalid: '\u041e\u0431\u043b\u0430\u043a\u043e \u0432\u0435\u0440\u043d\u0443\u043b\u043e \u043d\u0435\u0438\u0437\u0432\u0435\u0441\u0442\u043d\u044b\u0435 \u0434\u0430\u043d\u043d\u044b\u0435. \u0421\u0438\u043d\u0445\u0440\u043e\u043d\u0438\u0437\u0430\u0446\u0438\u044f \u043f\u0440\u0438\u043e\u0441\u0442\u0430\u043d\u043e\u0432\u043b\u0435\u043d\u0430.',
    syncUnknown: '\u041d\u0435\u0438\u0437\u0432\u0435\u0441\u0442\u043d\u0430\u044f \u043e\u0448\u0438\u0431\u043a\u0430 \u0441\u0438\u043d\u0445\u0440\u043e\u043d\u0438\u0437\u0430\u0446\u0438\u0438. \u041f\u043e\u0432\u0442\u043e\u0440\u0438\u0442\u0435 \u043f\u043e\u043f\u044b\u0442\u043a\u0443.'
  },
  es: {
    passwordFieldsRequired: 'Todos los campos de contrase\u00f1a son obligatorios.',
    passwordTooShort: 'La nueva contrase\u00f1a debe tener al menos 6 caracteres.',
    passwordMustDiffer: 'Elige una nueva contrase\u00f1a distinta de la actual.',
    passwordsDoNotMatch: 'Las nuevas contrase\u00f1as no coinciden.',
    passwordChanged: 'Contrase\u00f1a cambiada. Inicia sesi\u00f3n de nuevo.',
    staleRevisionAgain: 'Los datos volvieron a cambiar. Realiza la acci\u00f3n de nuevo.',
    targetChanged: 'Otro dispositivo modific\u00f3 o elimin\u00f3 este elemento. Se cargaron los datos m\u00e1s recientes.',
    syncNetworkRetrying: 'La conexi\u00f3n se interrumpi\u00f3. PixelDone reintenta autom\u00e1ticamente.',
    syncAuthExpired: 'Tu sesi\u00f3n caduc\u00f3. Inicia sesi\u00f3n de nuevo para reanudar la sincronizaci\u00f3n.',
    syncServerUpdateRequired: 'La versi\u00f3n de la nube es incompatible. Actualiza PixelDone e int\u00e9ntalo de nuevo.',
    syncLocalStorageError: 'No se pudieron guardar los datos de sincronizaci\u00f3n. Revisa el almacenamiento local.',
    syncRemoteDataInvalid: 'La nube devolvi\u00f3 datos desconocidos. La sincronizaci\u00f3n se ha pausado.',
    syncUnknown: 'Se produjo un error de sincronizaci\u00f3n desconocido. Int\u00e9ntalo de nuevo.'
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

export function windowsReliabilityMessage(
  locale: Locale,
  key: WindowsReliabilityMessageKey
): string {
  return reliabilityTranslations[locale][key];
}
