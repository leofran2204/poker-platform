-- 070: progresso de vídeo ligado à versão do vídeo (etapa 5 do plano de 25/11,
-- aulas da Academy em Remotion). O Pineapple Jogo Real (preparado, sem publicar) passa a ser a 071.
--
-- Cada linha passa a guardar o endereço do vídeo em que foi contada. Quando a aula
-- ganha um vídeo novo, o servidor zera a contagem e sorteia novas perguntas surpresa:
-- as antigas estavam marcadas em segundos do vídeo anterior e podiam cair depois do
-- fim do novo, deixando o aluno travado no meio da aula. Linhas sem endereço são
-- adotadas pelo vídeo atual.

ALTER TABLE course_watch ADD COLUMN IF NOT EXISTS video_url TEXT;

-- Estas 25 aulas trocam de vídeo nesta publicação (renderizador antigo -> Remotion),
-- então a contagem em andamento recomeça. Aula já concluída continua concluída
-- (course_progress não muda); m5l3-m5l5 mantêm o vídeo e a contagem.
DELETE FROM course_watch
WHERE video_url IS NULL
  AND lesson_id IN (
    'm0l1', 'm0l2', 'm0l3', 'm0l4', 'm0l5', 'm0l6', 'm0l7', 'm0l8',
    'm1l1', 'm1l2', 'm1l3', 'm2l1', 'm2l2', 'm2l3', 'm3l1', 'm3l2', 'm3l3',
    'm4l1', 'm4l2', 'm4l3', 'm5l1', 'm5l2', 'm6l1', 'm6l2', 'm6l3'
  );
