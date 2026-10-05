"""Original editorial animation. Sources/scripts: episodes.json and course product document.

Run with requirements.txt installed: python render.py --prepare / --episode home
Outputs: public videos, VTT, WebP poster, transcript and production manifest.
Narration is synthetic (pt-BR-AntonioNeural); no third-party music or slides.
"""
from __future__ import annotations

import argparse
import asyncio
import hashlib
import html
import json
import math
import unicodedata
from pathlib import Path
import re
import subprocess
import sys
import urllib.request
import wave

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / '.codex-tmp/media-deps'))
import edge_tts
import imageio_ffmpeg
import numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageOps

HERE = Path(__file__).resolve().parent
PUBLIC = ROOT / 'Frontend-Web/public'
WORK = HERE / 'media'
OUT = PUBLIC / 'videos'
FFMPEG = imageio_ffmpeg.get_ffmpeg_exe()
W, H, FPS = 1920, 1080, 30
BG, INK, GOLD, MUTED, GREEN = '#0b1413', '#f2eee4', '#d3b879', '#aabbb4', '#66c3a5'
FONTDIR = Path('C:/Windows/Fonts')

def font(size, bold=False, serif=False):
    name = 'georgia.ttf' if serif else ('segoeuib.ttf' if bold else 'segoeui.ttf')
    return ImageFont.truetype(str(FONTDIR / name), size)

def fit_text(draw, value, size, max_width, bold=False, serif=False):
    while size > 24 and max(draw.textlength(line, font=font(size,bold,serif)) for line in value.split('\n')) > max_width:
        size -= 2
    return font(size,bold,serif)

def prepare():
    assets = HERE / 'assets'
    assets.mkdir(exist_ok=True)
    hero = Image.open(assets / 'hero-source.png').convert('RGB')
    for width in (800, 1600):
        img = hero.copy()
        img.thumbnail((width, width))
        img.save(PUBLIC / f'brand/home/hero-{width}.webp', quality=85, method=6)
    met = assets / 'cloisters.jpg'
    if not met.exists():
        request = urllib.request.Request('https://collectionapi.metmuseum.org/api/collection/v1/iiif/475513/1545354/main-image', headers={'User-Agent':'ZeroTiltAcademy/1.0 (educational source research)'})
        with urllib.request.urlopen(request, timeout=60) as response:
            met.write_bytes(response.read())
    print('Assets ready', flush=True)

def stamp(sec):
    ms = round(sec * 1000)
    return f'{ms//3600000:02}:{ms//60000%60:02}:{ms//1000%60:02}.{ms%1000:03}'

def speech_text(value):
    return re.sub(r'[^a-z0-9]', '', unicodedata.normalize('NFKD', value.casefold()))

def complete_speech(voice, words):
    # Boundaries must cover the entire text, including its final words. A
    # successful network stream can still contain only the opening fragment.
    actual = speech_text(' '.join(word['text'] for word in words))
    expected = speech_text(voice)
    return bool(actual) and actual == expected

async def prepare_speech(key, i, scene, semaphore):
    digest=hashlib.sha256(scene['voice'].encode()).hexdigest()[:10]
    path=WORK / f'{key}-{i}-{digest}.mp3'
    bounds=path.with_suffix('.json')
    if path.exists() and bounds.exists():
        try:
            if complete_speech(scene['voice'], json.loads(bounds.read_text(encoding='utf-8'))):
                return path, bounds
        except (ValueError, KeyError):
            pass
    async with semaphore:
        for attempt in range(4):
            try:
                words=[]
                tts=edge_tts.Communicate(scene['voice'], 'pt-BR-AntonioNeural', rate='-3%', boundary='WordBoundary')
                temporary=path.with_suffix('.partial')
                with temporary.open('wb') as stream:
                    async for event in tts.stream():
                        if event['type']=='audio': stream.write(event['data'])
                        elif event['type']=='WordBoundary': words.append(event)
                if not complete_speech(scene['voice'], words):
                    raise RuntimeError(f'Incomplete narration: {key}/{i+1}')
                temporary.replace(path)
                bounds.write_text(json.dumps(words,ensure_ascii=False),encoding='utf-8')
                return path, bounds
            except Exception:
                if attempt == 3: raise
                await asyncio.sleep(1 + attempt)

async def narration(key, scenes):
    semaphore=asyncio.Semaphore(3)
    prepared=await asyncio.gather(*(prepare_speech(key,i,scene,semaphore) for i,scene in enumerate(scenes)))
    tracks=[]
    captions=[]
    start=0.0
    for i, scene in enumerate(scenes):
        path,bounds=prepared[i]
        words=json.loads(bounds.read_text(encoding='utf-8'))
        pcm=subprocess.check_output([FFMPEG,'-v','error','-i',str(path),'-f','s16le','-ac','1','-ar','48000','-'])
        samples=np.frombuffer(pcm,dtype='<i2').astype(np.float32)/32768
        lead=.25 if key=='pineapple' else .4
        tail=.55 if key=='pineapple' else .75
        length=math.ceil((len(samples)/48000+lead+tail)*FPS)/FPS
        scene['speechLead']=lead
        scene.update(start=start,duration=length)
        scene['words']=words
        track=np.zeros(round(length*48000),dtype=np.float32)
        track[round(lead*48000):round(lead*48000)+len(samples)]=samples
        tracks.append(track)
        group=[]
        for j,word in enumerate(words):
            group.append(word)
            text=' '.join(w['text'] for w in group)
            if len(text)>60 or len(group)>=10 or j==len(words)-1:
                a=start+lead+group[0]['offset']/1e7
                b=start+lead+(word['offset']+word['duration'])/1e7
                captions.append((a,b,text))
                group=[]
        start+=length
        print(f'Narration {key}/{i+1}: {length:.1f}s',flush=True)
    audio=np.concatenate(tracks)
    # Original, restrained sound design: a short felt-like tick at scene changes.
    rng=np.random.default_rng(7)
    for scene in scenes[1:]:
        n=round(.12*48000)
        tick=rng.normal(0,.015,n)*np.exp(-np.arange(n)/(48000*.018))
        offset=round(scene['start']*48000)
        audio[offset:offset+n]+=tick
    peak=np.max(np.abs(audio))
    if peak>0: audio=audio*.87/peak
    wav=WORK / f'{key}-narration.wav'
    with wave.open(str(wav),'wb') as stream:
        stream.setparams((1,2,48000,0,'NONE','not compressed'))
        stream.writeframes((np.clip(audio,-1,1)*32767).astype('<i2').tobytes())
    return wav,captions,start

def card(value, suit='♠', back=False):
    im=Image.new('RGBA',(230,326))
    d=ImageDraw.Draw(im)
    d.rounded_rectangle((2,2,226,322),radius=18,fill=('#173e33' if back else '#f2eee4'),outline=GOLD,width=3)
    if back:
        d.rounded_rectangle((16,16,212,308),radius=12,outline=GOLD,width=2)
        for y in range(45,285,35):
            for x in range(40,210,35):
                d.polygon([(x,y-9),(x+9,y),(x,y+9),(x-9,y)],outline='#688477')
    else:
        color='#a4373c' if suit in ('♥','♦') else '#15382f'
        d.text((22,15),value,font=font(47,True),fill=color)
        d.text((85,128),suit,font=font(72),fill=color)
        d.text((169,260),value,font=font(36,True),fill=color)
    return im

def scene_base(scene, idx, count):
    im=Image.new('RGB',(W,H),BG)
    kind=scene['kind']
    if kind=='hero':
        im=ImageOps.fit(Image.open(HERE/'assets/hero-source.png').convert('RGB'),(W,H))
        overlay=Image.new('RGBA',(W,H),(5,13,12,30))
        im=Image.alpha_composite(im.convert('RGBA'),overlay).convert('RGB')
    d=ImageDraw.Draw(im)
    d.line((100,108,1820,108),fill='#34433d',width=1)
    d.text((100,53),'ZT  /  ZERO TILT ACADEMY',font=font(24,True),fill=INK)
    d.text((1715,53),f'{idx+1:02} / {count:02}',font=font(23),fill=MUTED)
    d.text((100,192),scene['label'],font=font(25,True),fill=GOLD)
    max_width=1020 if kind in ('hero','museum','cards','kuhn') else 1670
    title_font=fit_text(d,scene['title'],92,max_width,serif=True)
    d.multiline_text((96,267),scene['title'],font=title_font,fill=INK,spacing=12)
    detail_y=540 if '\n' in scene['title'] else 430
    detail_font=fit_text(d,scene['detail'],31,max_width)
    d.multiline_text((100,detail_y),scene['detail'],font=detail_font,fill=MUTED,spacing=18)
    if kind=='museum':
        art=Image.open(HERE/'assets/cloisters.jpg').convert('RGB')
        art.thumbnail((570,650))
        x,y=1225+(570-art.width)//2,190+(650-art.height)//2
        d.rounded_rectangle((1200,165,1820,875),radius=12,fill='#e2dbc9')
        im.paste(art,(x,y))
        d.text((1225,899),'ACERVO THE MET · IMAGEM EM DOMÍNIO PÚBLICO',font=font(16),fill=MUTED)
    if kind=='question':
        d.ellipse((1500,530,1760,790),outline=GOLD,width=3)
        d.text((1590,570),'?',font=font(115,serif=True),fill=GOLD)
    source=scene.get('source','ZERO TILT POKER · NARRAÇÃO SINTÉTICA · 18+')
    d.text((100,975),source,font=fit_text(d,source,21,1700),fill=MUTED)
    return im

def animate(base,scene,t):
    im=base.copy().convert('RGBA')
    d=ImageDraw.Draw(im)
    progress=min(1,t/scene['duration'])
    ease=lambda delay: 1-(1-min(1,max(0,(t-delay)/1.1)))**3
    kind=scene['kind']
    if kind in ('cards','kuhn'):
        vals=['A','K','Q','J'] if kind=='cards' else ['1','2','3']
        for i,value in enumerate(vals):
            c=card(value,['♠','♥','♦','♣'][i],back=(kind=='kuhn' and i==2))
            c=c.resize((164,234),Image.Resampling.LANCZOS)
            angle=(i-(len(vals)-1)/2)*-9
            c=c.rotate(angle,resample=Image.Resampling.BICUBIC,expand=True)
            x=1120+i*133
            y=395+int(250*(1-ease(i*.25)))+int(4*math.sin(t*.6+i))
            im.alpha_composite(c,(x,y))
    if kind=='odds':
        for i,(value,label) in enumerate([('150','NO POTE'),('50','PARA PAGAR'),('25%','EQUILÍBRIO')]):
            x=100+i*560
            p=ease(i*.5)
            y=570+int(80*(1-p))
            d.rounded_rectangle((x,y,x+510,y+235),radius=16,fill='#15352c',outline='#456054',width=2)
            d.text((x+35,y+20),value,font=font(94,True),fill=GOLD if i==2 else INK)
            d.text((x+38,y+155),label,font=font(25,True),fill=MUTED)
        d.text((105,845),'50 ÷ (150 + 50) = 25% · Decisão final, sem rake ou apostas futuras.',font=font(27),fill=MUTED)
    if kind=='pineapple':
        for i,(label,count) in enumerate([('PRÉ-FLOP',2),('FLOP',3),('TURN',4),('RIVER',5)]):
            x=105+i*445
            d.text((x,640),label,font=font(24,True),fill=GOLD)
            for j in range(count):
                c=card('',back=True).resize((85,122),Image.Resampling.LANCZOS)
                im.alpha_composite(c,(x+j*62,695+int(65*(1-ease(i*.35)))))
            d.text((x,845),f'{count} cartas privadas',font=font(23),fill=MUTED)
    if kind=='shortdeck':
        for i,value in enumerate(['6','7','8','9','10','J','Q','K','A']):
            c=card(value).resize((120,170),Image.Resampling.LANCZOS)
            im.alpha_composite(c,(105+i*190,685+int(60*(1-ease(i*.1)))))
    if kind in ('timeline','journey'):
        items=scene['markers'] if kind=='timeline' else ['OBSERVE','PENSE','PRATIQUE']
        y=745
        d.line((115,y,1780,y),fill='#344b41',width=3)
        d.line((115,y,115+int(1665*ease(.25)),y),fill=GOLD,width=3)
        for i,item in enumerate(items):
            x=160+i*1430/max(1,len(items)-1)
            p=ease(.3+i*.3)
            if p>0:
                d.ellipse((x-9,y-9,x+9,y+9),fill=GOLD)
                d.text((x-50,y+35),item,font=font(32,True),fill=INK)
    d.rectangle((100,945,100+int(1720*progress),948),fill=GOLD)
    # Short fades keep scene cuts readable without decorative constant motion.
    fade=min(1,t/.3,(scene['duration']-t)/.3)
    if fade<1: im=Image.blend(Image.new('RGBA',(W,H),BG),im,max(0,fade))
    return im.convert('RGB')

# Aulas técnicas: roteiro deriva do material didático e dos exercícios. O hash
# impede publicar novamente um vídeo antigo depois de uma revisão do conteúdo.
ACADEMY_RENDER_VERSION = 3
COURSE_FILE = ROOT / 'Frontend-Web/src/data/courseContent.json'
TRAINING_FILE = ROOT / 'Frontend-Web/src/data/courseTraining.json'

def lesson_fingerprint(lesson, training):
    payload = {key: lesson.get(key, [] if key in ('quiz', 'sources') else '')
               for key in ('id', 'title', 'body', 'quiz', 'sources')}
    payload['training'] = [s for s in training['scenarios'] if s['lesson'] == lesson['id']]
    raw = json.dumps(payload, ensure_ascii=False, separators=(',', ':'))
    return hashlib.sha256(raw.encode('utf-8')).hexdigest()

def speak(text):
    """Expand card notation/formulas for Portuguese narration; no hidden text."""
    text = re.sub(r'[*`#]', '', text)
    text = re.sub(r'^\s*(?:\d+\.|-)\s*', '', text, flags=re.M)
    ranks = {'A':'ás','K':'rei','Q':'dama','J':'valete','T':'dez',
             '2':'dois','3':'três','4':'quatro','5':'cinco','6':'seis','7':'sete','8':'oito','9':'nove'}
    suits = {'h':'copas','d':'ouros','c':'paus','s':'espadas'}
    # Bare "As" is an article unless next to another concrete card.
    def expand_card(match):
        code = match.group()
        if code == 'As':
            context = text[max(0,match.start()-4):match.end()+5]
            if len(re.findall(r'\b[AKQJT2-9][shdc]\b', context)) < 2:
                return code
        return ranks[code[0]]+' de '+suits[code[1]]
    text = re.sub(r'\b[AKQJT2-9][shdc]\b', expand_card, text)
    def expand_sequence(match):
        values=[ranks[c] for c in match.group() if c in ranks]
        return ', '.join(values[:-1])+' e '+values[-1]
    text = re.sub(r'\b[AKQJT2-9](?:-[AKQJT2-9]){2,5}\b',expand_sequence,text)
    text = re.sub(r'\b(?=[AKQJT2-9]*[AKQJT])[AKQJT2-9]{3,5}\b',expand_sequence,text)
    def expand_hand(match):
        a,b,style=match.groups()
        pair='par de '+{'A':'ases','K':'reis','Q':'damas','J':'valetes','T':'dez'}.get(a,ranks[a])
        value=pair if a==b else ranks[a]+' e '+ranks[b]
        return value+({'s':' do mesmo naipe','o':' de naipes diferentes'}.get(style,''))
    text = re.sub(r'\b(?=[AKQJT2-9]*[AKQJTso])([AKQJT2-9])([AKQJT2-9])([so])?\b',expand_hand,text)
    text = re.sub(r'\bC\(([^,]+),([^\)]+)\)', r'combinação de \1 elementos tomados \2 a \2', text)
    for token, value in {'BB':'big blinds','BTN':'botão','UTG':'posição inicial','CO':'cutoff','SPR':'relação entre stack efetivo e pote','EV':'valor esperado','MDF':'frequência mínima de defesa','ZT':'Zê Tê'}.items():
        text = re.sub(r'\b'+token+r'\b', value, text)
    for token, value in [('→',' para '),('×',' vezes '),('÷',' dividido por '),('≈',' aproximadamente '),('=',' igual a '),('−',' menos '),('>',' maior que '),('+',' mais '),('%',' por cento'),('½',' meio ')]:
        text=text.replace(token,value)
    text = re.sub(r'(?<=\d)/(?=\d)', ' dividido por ', text)
    return re.sub(r'\s+', ' ', text).strip()

def wrap(draw, text, chosen_font, width, max_lines=None):
    lines=[]
    for paragraph in text.split('\n'):
        line=''
        for word in paragraph.split():
            candidate=(line+' '+word).strip()
            if line and draw.textlength(candidate,font=chosen_font)>width:
                lines.append(line); line=word
            else: line=candidate
        if line: lines.append(line)
    if max_lines is not None and len(lines)>max_lines:
        lines=lines[:max_lines]
        while draw.textlength(lines[-1]+'…',font=chosen_font)>width:
            lines[-1]=lines[-1].rsplit(' ',1)[0]
        lines[-1]+='…'
    return '\n'.join(lines)

def academy_scenes(lesson, training):
    related=[s for s in training['scenarios'] if s['lesson']==lesson['id']]
    scenario = next((s for s in related if s['board']),related[0])
    scenes=[{'title':lesson['title'],'label':'OBJETIVO DA AULA','text':scenario['objective'],
             'voice':speak(f"Zero Tilt Academy. {lesson['title']}. Nesta aula, nosso objetivo é: {scenario['objective']} Vamos desenvolver a teoria, resolver exemplos e aplicar a ideia na mesa deste módulo."),
             'diagram':'objective','hole':scenario['hole'],'board':scenario['board']}]
    sections = re.split(r'(?m)^#{2,3}\s+',lesson['body'])
    for section in sections:
        if not section.strip(): continue
        heading, _, body = section.partition('\n')
        if not body.strip(): continue
        body=re.sub(r'[*`]', '', body.strip())
        paragraphs=[p.strip() for p in body.split('\n\n') if p.strip()]
        # Chunk long paragraphs at sentence boundaries: a new, readable visual
        # every ~25–35 seconds instead of a wall of text held for minutes.
        chunks=[]
        for paragraph in paragraphs:
            current=[]
            for sentence in re.split(r'(?<=[.!?])\s+(?=[A-ZÁÉÍÓÚÀÂÊÔÃÕÇ“])',paragraph):
                current.append(sentence)
                if len(' '.join(current).split())>=65:
                    chunks.append(' '.join(current)); current=[]
            if current: chunks.append(' '.join(current))
        for part,chunk in enumerate(chunks):
            label='TEORIA E EXEMPLOS'
            if heading.startswith('Laboratório'): label='APLICAÇÃO NA MESA'
            diagram='cards'
            if any(word in heading.lower() for word in ['probabil','cont','denominador','preço','ev','combinaç','matemática']): diagram='math'
            if 'distribui' in heading.lower() or lesson['id']=='m5l3' and part==0: diagram='distribution'
            scenes.append({'title':heading,'label':label,'text':chunk,
                           'voice':speak((heading+'. ' if part==0 else '')+chunk),
                           'diagram':diagram,'hole':scenario['hole'],'board':scenario['board']})
    for i,q in enumerate(lesson['quiz']):
        correct=q['options'][q['expectedIndex']] if q['kind']=='theory' else q['expected']
        explanation=q['explanation'] if q['kind']=='theory' else q['why']
        context=''
        if q['kind']=='engine': context=f"Suas cartas: {', '.join(q['hole'])}. Cartas comunitárias: {', '.join(q['board']) or 'ainda não reveladas'}. "
        options = '\n'.join(f'{chr(65+j)}. {option}' for j,option in enumerate(q.get('options', [])))
        prompt = q['prompt'] + ('\n\n' + options if options else '')
        scenes.append({'title':f'Exercício resolvido {i+1}','label':'PENSE ANTES DA RESPOSTA',
            'text':prompt,'voice':speak(context+prompt+' Pause o vídeo, faça sua leitura e registre uma resposta antes de continuar.'),
            'diagram':'question','exerciseHand':q['kind']=='engine','hole':q.get('hole',scenario['hole']),'board':q.get('board',scenario['board'])})
        scenes.append({'title':'Confira o raciocínio','label':f'RESOLUÇÃO · EXERCÍCIO {i+1}',
            'text':correct+'\n\n'+explanation,'voice':speak('Resposta do exercício: '+correct+'. '+explanation),
            'diagram':'answer','exerciseHand':q['kind']=='engine','hole':q.get('hole',scenario['hole']),'board':q.get('board',scenario['board'])})
    scenes.append({'title':'Agora, pratique','label':'TEORIA → DECISÃO → REVISÃO',
        'text':f"{scenario['title']}\n\n1. Treino guiado\n2. Desafio sem dicas\n3. Replay e comparação\n4. Revisão em outra sessão",
        'voice':speak(f"Abra o simulador da aula e escolha o cenário {scenario['title']}. Jogue uma mão completa, responda às perguntas e registre seu plano. Depois use o replay para comparar outra ação na mesma distribuição. Volte em outra sessão no modo desafio. Avalie o raciocínio com as informações disponíveis, sem confundir uma vitória isolada com uma boa estratégia. As fontes estão no material de apoio. Este curso não promete lucro e o treino usa fichas sem valor."),
        'diagram':'practice','hole':scenario['hole'],'board':scenario['board']})
    return scenes

def academy_frame(lesson, scene, index, count, focus=0):
    im=Image.new('RGB',(W,H),BG)
    d=ImageDraw.Draw(im)
    d.rectangle((1150,0,W,H),fill='#10261f')
    d.line((88,112,1832,112),fill='#3c5147',width=2)
    d.text((88,55),'ZT / ACADEMY',font=font(26,True),fill=GOLD)
    d.text((1470,57),f'{lesson["id"].upper()}  ·  {index+1:02}/{count:02}',font=font(22),fill=MUTED)
    d.text((88,163),scene['label'],font=font(22,True),fill=GREEN)
    title_font=font(64,serif=True)
    title=wrap(d,scene['title'],title_font,970)
    if len(title.splitlines())>3:
        title_font=font(52,serif=True); title=wrap(d,scene['title'],title_font,970)
    d.multiline_text((84,222),title,font=title_font,fill=INK,spacing=10)
    y=247+len(title.splitlines())*(title_font.size+10)
    # Show consecutive excerpts, never truncate narration or the transcript.
    lines=wrap(d,scene['text'],font(33),945).splitlines()
    window=lines[focus*9:(focus+1)*9] or lines[:9]
    d.multiline_text((90,y+30),'\n'.join(window),font=font(33),fill='#cad6cf',spacing=15)
    kind=scene['diagram']
    variant=lesson.get('variant','holdem')
    variant_name={'holdem':'TEXAS HOLD’EM','short_deck':'TEXAS SHORT DECK','omaha':'OMAHA 4','brazilian_pineapple':'BRAZILIAN PINEAPPLE'}[variant]
    d.text((1200,175),variant_name,font=font(23,True),fill=GOLD)
    if kind=='distribution' and variant=='brazilian_pineapple':
        for i,(street,n) in enumerate([('PRÉ-FLOP',2),('FLOP',3),('TURN',4),('RIVER',5)]):
            sy=272+i*145
            d.text((1200,sy),street,font=font(24,True),fill=MUTED)
            for j in range(n):
                c=card('',back=True).resize((64,90),Image.Resampling.LANCZOS)
                im.paste(c,(1400+j*69,sy-15),c)
        d.text((1200,877),'SEM DESCARTE · EXATAMENTE 2 + 3',font=font(22,True),fill=GREEN)
    elif kind=='math':
        if variant in ('brazilian_pineapple','omaha'):
            d.text((1200,330),'C(h, 2) × C(b, 3)',font=font(55,serif=True),fill=INK)
            rows=[('FLOP','3'),('TURN','24'),('RIVER','100')] if variant=='brazilian_pineapple' else [('FLOP','6'),('TURN','24'),('RIVER','60')]
            for i,(label,value) in enumerate(rows):
                sy=460+i*100
                d.text((1210,sy),label,font=font(26),fill=MUTED)
                d.text((1600,sy-10),value,font=font(46,True),fill=GOLD)
            d.multiline_text((1200,822),'Candidatas legais\n≠ probabilidades independentes',font=font(26),fill=GREEN,spacing=10)
        elif variant=='short_deck':
            d.text((1200,315),'36 − 2 − 4 = 30',font=font(53,serif=True),fill=INK)
            d.multiline_text((1200,442),'BARALHO\n− SUAS PRIVADAS\n− BOARD DO TURN',font=font(28),fill=MUTED,spacing=26)
            d.text((1200,690),'5 / 30 ≈ 16,67%',font=font(52,True),fill=GOLD)
            d.multiline_text((1200,787),'Uma copa no river, com quatro\ncopas conhecidas e modelo uniforme.\nCompletar não garante vencer.',font=font(25),fill=MUTED,spacing=12)
        else:
            d.multiline_text((1200,290),'CUSTO DO CALL\n÷ POTE APÓS O CALL',font=font(34,True),fill=INK,spacing=22)
            d.text((1220,510),'50 / 200',font=font(72,serif=True),fill=GOLD)
            d.text((1320,618),'25%',font=font(112,True),fill=INK)
            d.multiline_text((1200,802),'Exemplo de preço · sem rake\nEquity depende de um modelo.\nConte apenas o pote elegível.',font=font(25),fill=MUTED,spacing=12)
    else:
        holes=scene['hole']; board=scene['board']
        d.text((1200,290),'SUAS CARTAS',font=font(24,True),fill=MUTED)
        for i,code in enumerate(holes):
            c=card('10' if code[0]=='T' else code[0],{'s':'♠','h':'♥','d':'♦','c':'♣'}[code[1]]).resize((106,151),Image.Resampling.LANCZOS)
            im.paste(c,(1200+i*121,345),c)
        d.text((1200,555),'COMUNITÁRIAS',font=font(24,True),fill=MUTED)
        for i in range(5):
            if i<len(board):
                code=board[i]; c=card('10' if code[0]=='T' else code[0],{'s':'♠','h':'♥','d':'♦','c':'♣'}[code[1]]).resize((106,151),Image.Resampling.LANCZOS)
                im.paste(c,(1200+i*121,610),c)
            else: d.rounded_rectangle((1200+i*121,610,1306+i*121,761),radius=9,outline='#426b55',width=2)
        rule='EXATAMENTE 2 PRIVADAS + 3 COMUNITÁRIAS' if variant in ('omaha','brazilian_pineapple') else 'MELHORES 5 · USE 0, 1 OU 2 PRIVADAS'
        d.text((1200,828),rule,font=fit_text(d,rule,24,630,True),fill=GREEN)
        reference='MÃO DO EXERCÍCIO' if scene.get('exerciseHand') else 'REFERÊNCIA DA MESA DO MÓDULO'
        d.text((1200,885),reference,font=font(21),fill=MUTED)
    d.line((88,962,1832,962),fill='#344f40',width=3)
    d.line((88,962,88+int(1744*(index+1)/count),962),fill=GOLD,width=3)
    d.text((88,990),'ZERO TILT POKER · NARRAÇÃO SINTÉTICA · MATERIAL REVISADO EM 28/09/2026 · 18+',font=font(19),fill=MUTED)
    return im

async def render_academy(lesson_id):
    WORK.mkdir(exist_ok=True); OUT.mkdir(exist_ok=True)
    course=json.loads(COURSE_FILE.read_text(encoding='utf-8'))
    training=json.loads(TRAINING_FILE.read_text(encoding='utf-8'))
    lesson=next(l for m in course['modules'] for l in m['lessons'] if l['id']==lesson_id)
    fingerprint=lesson_fingerprint(lesson,training)
    manifest_path=HERE/f'academy-{lesson_id}-manifest.json'
    name=f'zt-academy-{lesson_id}-v3'
    if manifest_path.exists():
        old=json.loads(manifest_path.read_text(encoding='utf-8'))
        if old.get('contentHash')==fingerprint and old.get('rendererVersion')==ACADEMY_RENDER_VERSION and all((OUT/f'{name}.{ext}').exists() for ext in ('mp4','vtt','webp')) and hashlib.sha256((OUT/f'{name}.mp4').read_bytes()).hexdigest()==old.get('sha256'):
            print(f'Current: {lesson_id}',flush=True); return
    scenes=academy_scenes(lesson,training)
    wav,captions,total=await narration(f'academy-{lesson_id}',scenes)
    frames=[]
    for i,scene in enumerate(scenes):
        # Long text is split across timed cards; narration stays continuous.
        probe=ImageDraw.Draw(Image.new('RGB',(W,H)))
        count=max(1,math.ceil(len(wrap(probe,scene['text'],font(33),945).splitlines())/9))
        for focus in range(count):
            file=WORK/f'{name}-{i:02}-{focus}.png'
            academy_frame(lesson,scene,i,len(scenes),focus).save(file)
            frames.append((file,scene['duration']/count))
    concat=WORK/f'{name}-frames.txt'
    concat.write_text(''.join(f"file '{file.as_posix()}'\nduration {duration:.6f}\n" for file,duration in frames)+f"file '{frames[-1][0].as_posix()}'\n",encoding='utf-8')
    target=OUT/f'{name}.mp4'
    encoded=WORK/f'{name}-encoded.mp4'
    subprocess.run([FFMPEG,'-y','-v','error','-filter_threads','1','-f','concat','-safe','0','-i',str(concat),'-i',str(wav),
        '-vf','fps=30,format=yuv420p','-c:v','libx264','-preset','veryfast','-tune','stillimage','-threads','2','-crf','22',
        '-af','loudnorm=I=-16:TP=-1.5:LRA=9','-c:a','aac','-b:a','128k','-movflags','+faststart','-t',str(total),str(encoded)],check=True)
    # Preserve the previous playable file if encoding fails or the source changes.
    current=json.loads(COURSE_FILE.read_text(encoding='utf-8'))
    current_lesson=next(l for m in current['modules'] for l in m['lessons'] if l['id']==lesson_id)
    current_training=json.loads(TRAINING_FILE.read_text(encoding='utf-8'))
    if lesson_fingerprint(current_lesson,current_training)!=fingerprint:
        raise RuntimeError(f'Lesson {lesson_id} changed during rendering: publication cancelled')
    encoded.replace(target)
    (OUT/f'{name}.vtt').write_text('WEBVTT\n\n'+'\n\n'.join(f'{stamp(a)} --> {stamp(b)}\n{text}' for a,b,text in captions)+'\n',encoding='utf-8')
    poster=academy_frame(lesson,scenes[0],0,len(scenes)); poster.thumbnail((1280,720)); poster.save(OUT/f'{name}.webp',quality=88)
    transcript='\n\n'.join(s['voice'] for s in scenes)
    (HERE/f'academy-{lesson_id}-transcript.txt').write_text(transcript+'\n',encoding='utf-8')
    manifest={'lessonId':lesson_id,'filename':name,'contentHash':fingerprint,'rendererVersion':ACADEMY_RENDER_VERSION,
        'durationSeconds':round(total,2),'width':W,'height':H,'fps':FPS,'bytes':target.stat().st_size,
        'voice':'pt-BR-AntonioNeural (synthetic)','chapters':[{'start':round(s['start'],2),'title':s['title']} for s in scenes],
        'sha256':hashlib.sha256(target.read_bytes()).hexdigest()}
    manifest_path.write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    # Read latest content so an interrupted batch cannot overwrite a later edit.
    current=json.loads(COURSE_FILE.read_text(encoding='utf-8'))
    current_lesson=next(l for m in current['modules'] for l in m['lessons'] if l['id']==lesson_id)
    if lesson_fingerprint(current_lesson,training)!=fingerprint:
        raise RuntimeError(f'Lesson {lesson_id} changed during rendering: publication cancelled')
    current_lesson['video']={'publicationStatus':'published','url':f'/videos/{name}.mp4','captionsUrl':f'/videos/{name}.vtt',
        'posterUrl':f'/videos/{name}.webp','durationSeconds':manifest['durationSeconds'],'transcript':transcript,
        'chapters':manifest['chapters'],'contentHash':fingerprint,'rendererVersion':ACADEMY_RENDER_VERSION}
    COURSE_FILE.write_text(json.dumps(current,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(f'Published locally: {lesson_id} · {total:.2f}s · {target.stat().st_size} bytes',flush=True)

def episode_fingerprint(episode):
    return hashlib.sha256(json.dumps(episode,ensure_ascii=False,separators=(',',':')).encode('utf-8')).hexdigest()


async def render(key):
    WORK.mkdir(exist_ok=True)
    OUT.mkdir(exist_ok=True)
    episode=json.loads((HERE/'episodes.json').read_text(encoding='utf-8'))[key]
    episode_hash=episode_fingerprint(episode)
    lesson_id={'ep11':'m0l1','ep12':'m0l2','ep13':'m0l3'}.get(key)
    content_hash=None
    if lesson_id:
        course=json.loads(COURSE_FILE.read_text(encoding='utf-8'))
        training=json.loads(TRAINING_FILE.read_text(encoding='utf-8'))
        lesson=next(l for m in course['modules'] for l in m['lessons'] if l['id']==lesson_id)
        content_hash=lesson_fingerprint(lesson,training)
    scenes=episode['scenes']
    wav,captions,total=await narration(key,scenes)
    if key == 'pineapple':
        await render_pineapple(episode, episode_hash, wav, captions, total)
        return
    name=episode['filename']
    silent=WORK/f'{name}-silent.mp4'
    cmd=[FFMPEG,'-y','-v','error','-f','rawvideo','-vcodec','rawvideo','-s',f'{W}x{H}','-pix_fmt','rgb24','-r',str(FPS),'-i','-','-an','-c:v','libx264','-preset','fast','-crf','22','-pix_fmt','yuv420p',str(silent)]
    process=subprocess.Popen(cmd,stdin=subprocess.PIPE)
    try:
        for i,scene in enumerate(scenes):
            base=scene_base(scene,i,len(scenes))
            for f in range(round(scene['duration']*FPS)):
                process.stdin.write(animate(base,scene,f/FPS).tobytes())
            print(f'Rendered {key}/{i+1}',flush=True)
        process.stdin.close()
        if process.wait()!=0: raise RuntimeError('Video encoder failed')
    except BaseException:
        process.kill()
        raise
    target=OUT/f'{name}.mp4'
    encoded=WORK/f'{name}-encoded.mp4'
    subprocess.run([FFMPEG,'-y','-v','error','-i',str(silent),'-i',str(wav),'-c:v','copy','-af','loudnorm=I=-16:TP=-1.5:LRA=9','-c:a','aac','-b:a','128k','-movflags','+faststart','-shortest',str(encoded)],check=True)
    current_episode=json.loads((HERE/'episodes.json').read_text(encoding='utf-8'))[key]
    if episode_fingerprint(current_episode)!=episode_hash:
        raise RuntimeError(f'Episode {key} changed during rendering: publication cancelled')
    if lesson_id:
        course=json.loads(COURSE_FILE.read_text(encoding='utf-8'))
        lesson=next(l for m in course['modules'] for l in m['lessons'] if l['id']==lesson_id)
        training=json.loads(TRAINING_FILE.read_text(encoding='utf-8'))
        if lesson_fingerprint(lesson,training)!=content_hash:
            raise RuntimeError(f'Lesson {lesson_id} changed during rendering: publication cancelled')
    encoded.replace(target)
    (OUT/f'{name}.vtt').write_text('WEBVTT\n\n'+'\n\n'.join(f'{stamp(a)} --> {stamp(b)}\n{text}' for a,b,text in captions)+'\n',encoding='utf-8')
    poster=scene_base(scenes[0],0,len(scenes))
    poster.thumbnail((1280,720))
    poster.save(OUT/f'{name}.webp',quality=88)
    transcript='\n\n'.join(s['voice'] for s in scenes)
    (HERE/f'{key}-transcript.txt').write_text(transcript+'\n',encoding='utf-8')
    manifest={'filename':name,'durationSeconds':round(total,2),'width':W,'height':H,'fps':FPS,'bytes':target.stat().st_size,'voice':'pt-BR-AntonioNeural (synthetic)','chapters':[{'start':round(s['start'],2),'title':s['title'].replace('\n',' ')} for s in scenes],
              'episodeHash':episode_hash,'sha256':hashlib.sha256(target.read_bytes()).hexdigest(),'rendererVersion':3}
    if lesson_id:
        manifest['contentHash']=content_hash
    (HERE/f'{key}-manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    if key == 'home':
        public_info = {**manifest, 'transcript': transcript}
        (ROOT/'Frontend-Web/src/data/homeFilm.json').write_text(json.dumps(public_info,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    elif lesson_id:
        lesson['video']={**lesson.get('video',{}),'publicationStatus':'published',
            'url':f'/videos/{name}.mp4','posterUrl':f'/videos/{name}.webp','captionsUrl':f'/videos/{name}.vtt',
            'durationSeconds':manifest['durationSeconds'],'chapters':manifest['chapters'],
            'transcript':transcript,'contentHash':content_hash,'rendererVersion':manifest['rendererVersion']}
        COURSE_FILE.write_text(json.dumps(course,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(manifest,ensure_ascii=False),flush=True)

async def render_pineapple(episode, episode_hash, wav, captions, total):
    import pineapple
    if total > episode['maxDurationSeconds']:
        raise RuntimeError(f'Film is {total:.2f}s; revise script to fit 480s without speeding speech')
    scenes=episode['scenes']; name=episode['filename']
    # All cue phrases must exist before encoding starts.
    for scene in scenes: pineapple.prepare(scene, sys.modules[__name__])
    silent=WORK/f'{name}-silent.mp4'
    cmd=[FFMPEG,'-y','-v','error','-f','rawvideo','-pix_fmt','rgb24','-s','1280x720',
         '-r',str(pineapple.SOURCE_FPS),'-i','-','-an','-vf','fps=30','-c:v','libx264',
         '-preset','veryfast','-threads','2','-crf','21','-pix_fmt','yuv420p',str(silent)]
    process=subprocess.Popen(cmd,stdin=subprocess.PIPE)
    chapter_list=[]; frame_count=0
    audit=ROOT/'artifacts/pineapple-film'; audit.mkdir(parents=True,exist_ok=True)
    try:
        for i,scene in enumerate(scenes):
            if not chapter_list or chapter_list[-1]['title']!=scene['chapter']:
                chapter_list.append({'start':round(scene['start'],2),'title':scene['chapter']})
            # Absolute frame edges prevent per-scene rounding drift.
            last=round((scene['start']+scene['duration'])*pineapple.SOURCE_FPS)
            while frame_count<last:
                t=max(0,frame_count/pineapple.SOURCE_FPS-scene['start'])
                process.stdin.write(pineapple.frame(scene,t,i,len(scenes),sys.modules[__name__]).tobytes())
                frame_count+=1
            for label,t in [('start',2.5),('end',scene['duration']-1)]:
                pineapple.frame(scene,t,i,len(scenes),sys.modules[__name__]).save(audit/f'scene-{i+1:02}-{label}.png')
            print(f'Rendered pineapple/{i+1}: {scene["title"]}',flush=True)
        process.stdin.close()
        if process.wait()!=0: raise RuntimeError('Pineapple video encoder failed')
    except BaseException:
        process.kill(); process.wait(); raise
    encoded=WORK/f'{name}-encoded.mp4'
    # Add native MP4 chapters in addition to accessible HTML chapter buttons.
    metadata=WORK/f'{name}-chapters.txt'
    metadata.write_text(';FFMETADATA1\n'+''.join(
        f"[CHAPTER]\nTIMEBASE=1/1000\nSTART={round(c['start']*1000)}\nEND={round((chapter_list[j+1]['start'] if j+1<len(chapter_list) else total)*1000)}\ntitle={c['title']}\n"
        for j,c in enumerate(chapter_list)),encoding='utf-8')
    subprocess.run([FFMPEG,'-y','-v','error','-i',str(silent),'-i',str(wav),'-i',str(metadata),
        '-map','0:v:0','-map','1:a:0','-map_metadata','2','-map_chapters','2','-c:v','copy',
        '-af','loudnorm=I=-16:TP=-1.5:LRA=9','-c:a','aac','-b:a','128k','-movflags','+faststart',
        '-t',str(total),str(encoded)],check=True)
    current=json.loads((HERE/'episodes.json').read_text(encoding='utf-8'))['pineapple']
    if episode_fingerprint(current)!=episode_hash: raise RuntimeError('Pineapple script changed during rendering')
    target=OUT/f'{name}.mp4'; encoded.replace(target)
    (OUT/f'{name}.vtt').write_text('WEBVTT\n\n'+'\n\n'.join(f'{stamp(a)} --> {stamp(b)}\n{text}' for a,b,text in captions)+'\n',encoding='utf-8',newline='\n')
    pineapple.poster(sys.modules[__name__]).save(OUT/f'{name}.webp',quality=92)
    transcript='\n\n'.join(s['voice'] for s in scenes)
    transcript_path=HERE/'pineapple-transcript.txt'; transcript_path.write_text(transcript+'\n',encoding='utf-8',newline='\n')
    manifest={'filename':name,'title':episode['title'],'bettingRuleVersion':episode['bettingRuleVersion'],
        'durationSeconds':round(total,2),'width':1280,'height':720,'fps':30,'bytes':target.stat().st_size,
        'voice':'pt-BR-AntonioNeural (synthetic)','voiceRate':'-3%','chapters':chapter_list,
        'sha256':hashlib.sha256(target.read_bytes()).hexdigest(),'episodeHash':episode_hash,'rendererVersion':1,
        'assetHashes':{ext:hashlib.sha256((OUT/f'{name}.{ext}').read_bytes()).hexdigest() for ext in ('mp4','vtt','webp')},
        'transcriptHash':hashlib.sha256(transcript_path.read_bytes()).hexdigest(),
        'scenes':[{'start':round(s['start'],3),'duration':round(s['duration'],3),'title':s['title'],
                   'cueTime':s['cueTime'],'finalCueTime':s['finalCueTime'],'highlights':s['highlights']} for s in scenes]}
    (HERE/'pineapple-manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    # Scene timings belong to the production manifest, not the initial page bundle.
    public={k:v for k,v in manifest.items() if k!='scenes'}
    (ROOT/'Frontend-Web/src/data/pineappleFilm.json').write_text(json.dumps({**public,'transcript':transcript},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(f'Pineapple ready locally: {total:.2f}s / {target.stat().st_size} bytes',flush=True)

def check_publications(decode=False, ready_only=False, only=None):
    """Validate active media only. --decode also reads every audio/video frame."""
    course=json.loads(COURSE_FILE.read_text(encoding='utf-8'))
    training=json.loads(TRAINING_FILE.read_text(encoding='utf-8'))
    entries=[]
    for module in course['modules']:
        for lesson in module['lessons']:
            video=lesson.get('video',{})
            prior_rules = video.get('publicationStatus') == 'prior_rules'
            if video.get('publicationStatus') not in ('published', 'prior_rules'):
                if ready_only: continue
                raise RuntimeError(f"Lesson without current video: {lesson['id']}")
            if module['id']!='m0':
                manifest=json.loads((HERE/f"academy-{lesson['id']}-manifest.json").read_text(encoding='utf-8'))
                if prior_rules:
                    assert lesson['id'] in ('m5l3', 'm5l4', 'm5l5'), lesson['id']
                    assert video['bettingRuleVersion']==manifest['bettingRuleVersion']=='legacy_no_limit', lesson['id']
                    original=json.dumps(manifest['sourceSnapshot'], ensure_ascii=False, separators=(',', ':'))
                    assert video['contentHash']==hashlib.sha256(original.encode('utf-8')).hexdigest(), lesson['id']
                    assert video['contentHash']!=lesson_fingerprint(lesson,training), lesson['id']
                else:
                    assert video['contentHash']==lesson_fingerprint(lesson,training), lesson['id']
                assert manifest['contentHash']==video['contentHash'], lesson['id']
                assert manifest['rendererVersion']==video['rendererVersion']==ACADEMY_RENDER_VERSION, lesson['id']
            else:
                episode_key=f"ep{10+int(lesson['id'][-1])}"
                manifest=json.loads((HERE/f'{episode_key}-manifest.json').read_text(encoding='utf-8'))
                if manifest.get('episodeHash'):
                    episode=json.loads((HERE/'episodes.json').read_text(encoding='utf-8'))[episode_key]
                    assert manifest['episodeHash']==episode_fingerprint(episode), lesson['id']
                    assert manifest['contentHash']==video['contentHash']==lesson_fingerprint(lesson,training), lesson['id']
            entries.append((lesson['id'],video,manifest))
    home=json.loads((ROOT/'Frontend-Web/src/data/homeFilm.json').read_text(encoding='utf-8'))
    entries.append(('home',{'url':f"/videos/{home['filename']}.mp4",'captionsUrl':f"/videos/{home['filename']}.vtt",'posterUrl':f"/videos/{home['filename']}.webp",**home},home))
    pineapple=json.loads((ROOT/'Frontend-Web/src/data/pineappleFilm.json').read_text(encoding='utf-8'))
    manifest=json.loads((HERE/'pineapple-manifest.json').read_text(encoding='utf-8'))
    episode=json.loads((HERE/'episodes.json').read_text(encoding='utf-8'))['pineapple']
    assert manifest['episodeHash']==pineapple['episodeHash']==episode_fingerprint(episode)
    assert manifest['bettingRuleVersion']==pineapple['bettingRuleVersion']==episode['bettingRuleVersion']=='brazilian_pineapple_hybrid_v1'
    assert 0<pineapple['durationSeconds']<=episode['maxDurationSeconds']==480
    assert manifest['transcriptHash']==hashlib.sha256((HERE/'pineapple-transcript.txt').read_bytes()).hexdigest()
    assert pineapple['transcript']==(HERE/'pineapple-transcript.txt').read_text(encoding='utf-8').strip()
    for ext,digest in manifest['assetHashes'].items():
        assert digest==pineapple['assetHashes'][ext]==hashlib.sha256((OUT/f"{pineapple['filename']}.{ext}").read_bytes()).hexdigest()
    entries.append(('pineapple',{'url':f"/videos/{pineapple['filename']}.mp4",'captionsUrl':f"/videos/{pineapple['filename']}.vtt",'posterUrl':f"/videos/{pineapple['filename']}.webp",**pineapple},manifest))
    if only: entries=[entry for entry in entries if entry[0]==only]
    def seconds(value):
        h,m,s=value.split(':'); return int(h)*3600+int(m)*60+float(s)
    records=[]
    for key,video,manifest in entries:
        target=PUBLIC/video['url'].lstrip('/')
        assert target.is_file() and target.stat().st_size==manifest['bytes'], key
        if manifest.get('sha256'):
            assert hashlib.sha256(target.read_bytes()).hexdigest()==manifest['sha256'], key
        assert abs(video['durationSeconds']-manifest['durationSeconds'])<.02, key
        assert video['chapters']==manifest['chapters'], key
        starts=[chapter['start'] for chapter in video['chapters']]
        assert starts==sorted(set(starts)) and starts[0]==0 and starts[-1]<video['durationSeconds'], key
        poster=PUBLIC/video['posterUrl'].lstrip('/')
        with Image.open(poster) as im: assert im.size==(1280,720), key
        vtt=(PUBLIC/video['captionsUrl'].lstrip('/')).read_text(encoding='utf-8')
        captions=re.findall(r'(\d{2}:\d{2}:\d{2}\.\d{3}) --> (\d{2}:\d{2}:\d{2}\.\d{3})\n([^\n]+)',vtt)
        previous=0
        for a,b,_ in captions:
            assert previous<=seconds(a)<seconds(b)<=video['durationSeconds']+.05, key
            previous=seconds(b)
        actual=speech_text(html.unescape(' '.join(c[2] for c in captions)))
        assert actual==speech_text(video['transcript']), f'{key}: captions/transcript differ'
        if decode:
            subprocess.run([FFMPEG,'-v','error','-xerror','-threads','2','-i',str(target),'-f','null','-'],check=True)
        records.append({'id':key,'seconds':video['durationSeconds'],'bytes':target.stat().st_size,'captions':len(captions),'decoded':decode})
        print(f'Checked {key}: {len(captions)} captions, {video["durationSeconds"]:.2f}s',flush=True)
    audit=ROOT/'artifacts/academy-audit'
    audit.mkdir(parents=True,exist_ok=True)
    report=f'media-{only}.json' if only else ('media-partial.json' if ready_only else 'media.json')
    (audit/report).write_text(json.dumps(records,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(f'Current publications: {len(entries)} · {sum(v[1]["durationSeconds"] for v in entries)/60:.1f} minutes',flush=True)

if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--prepare',action='store_true')
    parser.add_argument('--episode',choices=('home','pineapple','ep11','ep12','ep13','all'))
    parser.add_argument('--academy',help='Lesson ID or all (25 technical lessons).')
    parser.add_argument('--check',action='store_true',help='Validate all active lessons and the home film.')
    parser.add_argument('--decode',action='store_true',help='With --check, decode every media file completely.')
    parser.add_argument('--only',choices=('pineapple',),help='With --check, limit the media audit to the new film.')
    args=parser.parse_args()
    if args.prepare: prepare()
    if args.episode:
        for key in (('home','pineapple','ep11','ep12','ep13') if args.episode=='all' else (args.episode,)):
            asyncio.run(render(key))
    if args.academy:
        course=json.loads(COURSE_FILE.read_text(encoding='utf-8'))
        ids=[l['id'] for m in course['modules'] if m['id']!='m0' for l in m['lessons']]
        if args.academy!='all':
            if args.academy not in ids: parser.error('Unknown technical lesson')
            ids=[args.academy]
        for lesson_id in ids: asyncio.run(render_academy(lesson_id))
    if args.only and not args.check: parser.error('--only requires --check')
    if args.check: check_publications(args.decode,only=args.only)
