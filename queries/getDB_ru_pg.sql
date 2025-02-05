WITH t AS (
	SELECT
	c.relname as name
	,a.attname as cname
	,pg_catalog.format_type(a.atttypid, a.atttypmod) as ctype
	--,a.attnum
	
	FROM pg_catalog.pg_attribute a
	   JOIN pg_catalog.pg_class c ON a.attrelid = c.oid
		 JOIN pg_catalog.pg_namespace n ON c.relnamespace = n.oid
	
	WHERE
		a.attnum > 0
		AND NOT a.attisdropped
		AND n.nspname = 'public'
		AND c.relkind <> 'i'
		AND c.relname !~* 'chngr'
	)
select
    DataType,
    TableName,
    FieldName,
    TablePrefix,
    TableNumber,
    TableSuffix,
    VTPrefix,
    VTNumber,
    VTSuffix,
    FieldPrefix,
    FieldNumber,
    FieldSuffix
    --,dense_rank() over(partition by TablePrefix, TableNumber, TableSuffix, VTPrefix, VTNumber, VTSuffix order by TableName desc) as DN
  from (
    -- Константа
    select 'Const' AS DataType
          ,t.name AS TableName
          ,t.cname AS FieldName
          ,'Константа.' AS TablePrefix --No operator matches the given name and argument types. You might need to add explicit type casts. 
          ,substring(t.name, '[0-9]+') AS TableNumber
          ,'' AS TableSuffix
          ,'' AS VTPrefix
          ,'' AS VTNumber
          ,'' AS VTSuffix
          ,'' AS FieldPrefix
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end AS FieldNumber
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end AS FieldSuffix
    from t
        
    where 
      t.name ~* '[_]Const[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- Перечисление
    select 'Enum'
          ,t.name
          ,t.cname
          ,'Перечисление.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,t.cname
          , ''
    from t
        
    where 
      t.name ~* '[_]Enum[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- ПланОбмена
    select 'Node'
          ,t.name
          ,t.cname
          ,'ПланОбмена.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Node[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- ПланОбмена.ТабличнаяЧасть
    select 'VT'
          ,t.name
          ,t.cname
          ,'ПланОбмена.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,'.ТабличнаяЧасть.'
          ,substring(t.name, '_VT|_vt([0-9]+)')
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 5) = '_node' and right(t.cname, 7) = '_idrref' then right(t.cname, 7)
            when left(t.cname, 7) = '_lineno' then left(t.cname, 7)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Node[0-9].*[_]VT[0-9]'
    union all

    -- БизнесПроцессы
    select 'BPr'
          ,t.name
          ,t.cname
          ,'БизнесПроцесс.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]BPr[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- БизнесПроцессы.ТочкиМаршрута
    select 'BPrPoints'
          ,t.name
          ,t.cname
          ,'БизнесПроцесс.'
          ,substring(t.name, '[0-9]+')
          ,'.ТочкиМаршрута'
          ,''
          ,''
          ,''
          ,''
          ,t.cname
          , ''
    from t
        
    where 
      t.name ~* '[_]BPrPoints[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- БизнесПроцессы.ТабличнаяЧасть
    select 'VT'
          ,t.name
          ,t.cname
          ,'БизнесПроцесс.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,'.ТабличнаяЧасть.'
          ,substring(t.name, '_VT|_vt([0-9]+)')
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 4) = '_BPr' and right(t.cname, 7) = '_idrref' then right(t.cname, 7)
            when left(t.cname, 7) = '_lineno' then left(t.cname, 7)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]BPr[0-9].*[_]VT[0-9]'
    union all

    -- Задачи
    select 'Task'
          ,t.name
          ,t.cname
          ,'Задача.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 16) = '_businessprocess' then left(t.cname, 16)
            when left(t.cname, 6) = '_point' then left(t.cname, 6)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Task[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- Задачи.ТабличнаяЧасть
    select 'VT'
          ,t.name
          ,t.cname
          ,'Задача.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,'.ТабличнаяЧасть.'
          ,substring(t.name, '_VT|_vt([0-9]+)')
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 5) = '_task' and right(t.cname, 7) = '_idrref' then right(t.cname, 7)
            when left(t.cname, 7) = '_lineno' then left(t.cname, 7)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Task[0-9].*[_]VT[0-9]'
    union all

    -- ПланВидовХарактеристик
    select 'Chrc'
          ,t.name
          ,t.cname
          ,'ПланВидовХарактеристик.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 8) = '_ownerid' then left(t.cname, 8)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Chrc[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- ПланВидовХарактеристик.ТабличнаяЧасть
    select 'VT'
          ,t.name
          ,t.cname
          ,'ПланВидовХарактеристик.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,'.ТабличнаяЧасть.'
          ,substring(t.name, '_VT|_vt([0-9]+)')
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 5) = '_chrc' and right(t.cname, 7) = '_idrref' then right(t.cname, 7)
            when left(t.cname, 7) = '_lineno' then left(t.cname, 7)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Chrc[0-9].*[_]VT[0-9]'
    union all

    -- Справочник
    select 'Reference'
          ,t.name
          ,t.cname
          ,'Справочник.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 8) = '_ownerid' then left(t.cname, 8)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Reference[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- Справочник.ТабличнаяЧасть
    select 'VT'
          ,t.name
          ,t.cname
          ,'Справочник.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,'.ТабличнаяЧасть.'
          ,substring(t.name, '_VT|_vt([0-9]+)')
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 10) = '_reference' and right(t.cname, 7) = '_idrref' then right(t.cname, 7)
            when left(t.cname, 7) = '_lineno' then left(t.cname, 7)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Reference[0-9].*[_]VT[0-9]'
    union all

    -- Документ
    select 'Document'
          ,t.name
          ,t.cname
          ,'Документ.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Document[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- Документ.ТабличнаяЧасть
    select 'VT'
          ,t.name
          ,t.cname
          ,'Документ.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,'.ТабличнаяЧасть.'
          ,substring(t.name, '_VT|_vt([0-9]+)')
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 9) = '_document' and right(t.cname, 7) = '_idrref' then right(t.cname, 7)
            when left(t.cname, 7) = '_lineno' then left(t.cname, 7)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Document[0-9].*[_]VT[0-9]'
    union all

    -- ЖурналДокументов
    select 'DocumentJournal'
          ,t.name
          ,t.cname
          ,'ЖурналДокументов.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]DocumentJournal[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- РегистрСведений
    select 'InfoRg'
          ,t.name
          ,t.cname
          ,'РегистрСведений.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]InfoRg[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- РегистрСведений.ИтогиСрезПоследних
    select 'InfoRgSL'
          ,t.name
          ,t.cname
          ,'РегистрСведений.'
          ,substring(t.name, '[0-9]+')
          ,'.ИтогиСрезПоследних'
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]InfoRgSL[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- РегистрСведений.ИтогиСрезПервых
    select 'InfoRgSF'
          ,t.name
          ,t.cname
          ,'РегистрСведений.'
          ,substring(t.name, '[0-9]+')
          ,'.ИтогиСрезПервых'
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]InfoRgSF[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- РегистрНакопления
    select 'AccumRg'
          ,t.name
          ,t.cname
          ,'РегистрНакопления.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]AccumRg[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- РегистрНакопления.Остатки
    select 'AccumRgT'
          ,t.name
          ,t.cname
          ,'РегистрНакопления.'
          ,substring(t.name, '[0-9]+')
          ,'.Остатки'
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]AccumRgT[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- РегистрНакопления.Обороты
    select 'AccumRgTn'
          ,t.name
          ,t.cname
          ,'РегистрНакопления.'
          ,substring(t.name, '[0-9]+')
          ,'.Обороты'
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]AccumRgTn[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- ПланСчетов
    select 'Acc'
          ,t.name
          ,t.cname
          ,'ПланСчетов.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Acc[0-9]'
      and t.name !~*  '[_]ExtDim[0-9]'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- ПланСчетов.ВидыСубконто
    select 'ExtDim'
          ,t.name
          ,t.cname
          ,'ПланСчетов.'
          ,regexp_substr(t.name, '_ExtDim([0-9]+)',1,1,'i',1)
          ,'.ВидыСубконто'
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 4) = '_acc' and right(t.cname, 7) = '_idrref' then right(t.cname, 7)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Acc[0-9].*[_]ExtDim[0-9]'
    union all

    -- ПланСчетов.ТабличнаяЧасть
    select 'VT'
          ,t.name
          ,t.cname
          ,'ПланСчетов.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,'.ТабличнаяЧасть.'
          ,substring(t.name, '_VT|_vt([0-9]+)')
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 4) = '_acc' and right(t.cname, 7) = '_idrref' then right(t.cname, 7)
            when left(t.cname, 7) = '_lineno' then left(t.cname, 7)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]Acc[0-9].*[_]VT[0-9]'
    union all

    -- РегистрБухгалтерии
    select 'AccRg'
          ,t.name
          ,t.cname
          ,'РегистрБухгалтерии.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 8) = '_valuect' then '_valuesubcontoct'
            when left(t.cname, 8) = '_valuedt' then '_valuesubcontodt'
            when left(t.cname, 6) = '_value' then '_valuesubconto'
            when left(t.cname, 7) = '_kindct' then '_kindsubcontoct'
            when left(t.cname, 7) = '_kinddt' then '_kindsubcontodt'
            when left(t.cname, 5) = '_kind' then '_kindsubconto'
            else t.cname
          end
          ,
          case
            when left(t.cname, 4) = '_fld' then (case regexp_substr(t.cname, '_fld([0-9]+)(..)?',1,1,'i',2) when 'ct' then 'Кт' when 'dt' then 'Дт' else '' end)
            when left(t.cname, 8) = '_valuect' then substring(t.cname, '[0-9]+')
            when left(t.cname, 8) = '_valuedt' then substring(t.cname, '[0-9]+')
            when left(t.cname, 6) = '_value' then substring(t.cname, '[0-9]+')
            when left(t.cname, 7) = '_kindct' then substring(t.cname, '[0-9]+')
            when left(t.cname, 7) = '_kinddt' then substring(t.cname, '[0-9]+')
            when left(t.cname, 5) = '_kind' then substring(t.cname, '[0-9]+')
            else ''
          end ||
          case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]AccRg[0-9]'
    union all

    -- РегистрБухгалтерии.ЗначенияСубконто
    select 'AccRgED'
          ,t.name
          ,t.cname
          ,'РегистрБухгалтерии.'
          ,substring(t.name, '[0-9]+')
          ,'.ЗначенияСубконто'
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 6) = '_value' then left(t.cname, 6)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]AccRgED[0-9]'
    union all

    -- РегистрБухгалтерии.ИтогиМеждуСчетами
    select 'AccRgCT'
          ,t.name
          ,t.cname
          ,'РегистрБухгалтерии.'
          ,substring(t.name, '[0-9]+')
          ,'.ИтогиМеждуСчетами'
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          ,
          case
             when left(t.cname, 4) = '_fld' then (case regexp_substr(t.cname, '_fld([0-9]+)(..)?',1,1,'i',2) when 'ct' then 'Кт' when 'dt' then 'Дт' else '' end)
             else ''
          end ||
          case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]AccRgCT[0-9]'
    union all

    -- РегистрБухгалтерии.ИтогиПоСчетам
    select 'AccRgAT'
          ,t.name
          ,t.cname
          ,'РегистрБухгалтерии.'
          ,substring(t.name, '[0-9]+')
          ,case substring(t.name, 9, 1) when '0' then '.ИтогиПоСчетам' else '.ИтогиПоСчетамССубконто' || substring(t.name, 9, 1) end
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 6) = '_value' then '_valuesubconto'
            when left(t.cname, 11) = '_turnoverct' then substring(t.cname, '[0-9]+')
            when left(t.cname, 11) = '_turnoverdt' then substring(t.cname, '[0-9]+')
            when left(t.cname, 9) = '_turnover' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          ,
          case
            when left(t.cname, 6) = '_value' then substring(t.cname, '[0-9]+')
            when left(t.cname, 11) = '_turnoverct' then 'Кт'
            when left(t.cname, 11) = '_turnoverdt' then 'Дт'
            else ''
          end ||
          case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]AccRgAT[0-9]'
    union all

    -- ПланВидовРасчета
    select 'CKinds'
          ,t.name
          ,t.cname
          ,'ПланВидовРасчета.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]CKinds[0-9]'
      and t.name !~*  '[_]LeadingCK'
      and t.name !~*  '[_]BaseCK'
      and t.name !~*  '[_]DisplacedCK'
      and t.name !~*  '[_]VT[0-9]'
    union all

    -- ПланВидовРасчета.ВедущиеВидыРасчета
    select 'CKinds'
          ,t.name
          ,t.cname
          ,'ПланВидовРасчета.'
          ,substring(t.name, '[0-9]+')
          ,'.ВедущиеВидыРасчета'
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 7) = '_ckinds' and right(t.cname, 7) = '_idrref' then right(t.cname, 7)
            when left(t.cname, 19) = '_leadingckleadingck' then left(t.cname, 19)
            when t.cname = '_leadingcklineno' then '_lineno'
            when t.cname = '_predefinedleadingcktableline' then '_predefinedid'
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]CKinds[0-9].*[_]LeadingCK'
    union all

    -- ПланВидовРасчета.БазовыеВидыРасчета
    select 'CKinds'
          ,t.name
          ,t.cname
          ,'ПланВидовРасчета.'
          ,substring(t.name, '[0-9]+')
          ,'.БазовыеВидыРасчета'
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 7) = '_ckinds' and right(t.cname, 7) = '_idrref' then right(t.cname, 7)
            when left(t.cname, 13) = '_baseckbaseck' then left(t.cname, 13)
            when t.cname = '_basecklineno' then '_lineno'
            when t.cname = '_predefinedbasecktableline' then '_predefinedid'
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]CKinds[0-9].*[_]BaseCK'
    union all

    -- ПланВидовРасчета.ВытесняющиеВидыРасчета
    select 'CKinds'
          ,t.name
          ,t.cname
          ,'ПланВидовРасчета.'
          ,substring(t.name, '[0-9]+')
          ,'.ВытесняющиеВидыРасчета'
          ,''
          ,''
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 7) = '_ckinds' and right(t.cname, 7) = '_idrref' then right(t.cname, 7)
            when left(t.cname, 19) = '_displacedckdisplck' then left(t.cname, 19)
            when t.cname = '_displacedcklineno' then '_lineno'
            when t.cname = '_predefineddisplacedcktableline' then '_predefinedid'
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]CKinds[0-9].*[_]DisplacedCK'
    union all

    -- ПланВидовРасчета.ТабличнаяЧасть
    select 'VT'
          ,t.name
          ,t.cname
          ,'ПланВидовРасчета.'
          ,substring(t.name, '[0-9]+')
          ,''
          ,'.ТабличнаяЧасть.'
          ,substring(t.name, '_VT|_vt([0-9]+)')
          ,''
          ,''
          ,case
            when left(t.cname, 4) = '_fld' then substring(t.cname, '[0-9]+')
            when left(t.cname, 7) = '_ckinds' and right(t.cname, 7) = '_idrref' then right(t.cname, 7)
            when left(t.cname, 7) = '_lineno' then left(t.cname, 7)
            else t.cname
          end
          , case
            when right(t.cname, 5) = '_type' then '.Тип'
            when right(t.cname, 2) = '_l' then '.Булево'
            when right(t.cname, 2) = '_n' then '.Число'
            when right(t.cname, 2) = '_t' then '.Дата'
            when right(t.cname, 2) = '_s' then '.Строка'
            when right(t.cname, 2) = '_b' then '.Двоичный'
            when right(t.cname, 6) = '_rtref' then '.ВидСсылки'
            when right(t.cname, 6) = '_rrref' then '.Ссылка'
            else ''
          end
    from t
        
    where 
      t.name ~* '[_]CKinds[0-9].*[_]VT[0-9]'
  ) t
--) t
--where DN = 1
order by TableNumber
        ,TablePrefix
        ,TableSuffix
        ,VTNumber
        ,VTPrefix
        ,VTSuffix
        ,FieldNumber
        ,FieldPrefix
        ,FieldSuffix
