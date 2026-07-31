// Decompiled from QT2025 QuickTranslator.exe (ilspycmd 8.2.0.7535) — class LocNameOff
// and the threshold helper only. The full GUI decompile is NOT committed because it
// embeds third-party credentials; reproduce it with docs/dev/decompile.md if needed.

	public class LocNameOff
	{
		private static readonly Regex regexExtractPhrases = new Regex("《.*?》", RegexOptions.Compiled);

		private readonly string[] source = "她/着/的/你/我/了/他/什么/也/什/们/在/您/那/这/这个/不过/尔/啊/吧/一边/没/哪个/就是/有些/很/非常/还是/再有/发现/数/十几".Split(new char[1] { '/' });

		private static readonly JiebaSegmenter jiebaSegmenter = new JiebaSegmenter();

		public List<string> LocNameQT(string textContent, int threshold)
		{
			List<string> segments = (from w in jiebaSegmenter.Cut(textContent)
				select w.Trim()).ToList();
			HashSet<string> phrases = new HashSet<string>((from w in segments
				where w.Length >= 2 && w.Length <= 5
				group w by w into g
				where g.Count() >= threshold
				select g).ToDictionary((IGrouping<string, string> g) => g.Key, (IGrouping<string, string> g) => g.Count()).Keys.Where((string k) => !string.IsNullOrWhiteSpace(k) && !ContainsNumber(k) && !source.Any((string s) => k.Contains(s))));
			phrases = FilterUnnecessaryPhrasesOptimized(phrases);
			HashSet<string> validTerms = FilterPhrasesAndNames(phrases, textContent);
			ValidateAndMergeTerms(validTerms, segments);
			List<string> list = (from x in validTerms.Select(delegate(string key)
				{
					string formatted = BuildFormattedHanViet(key);
					return new
					{
						Key = key,
						Formatted = formatted
					};
				})
				where !global::TranslatorEngine.TranslatorEngine.vietPhraseDictionary.ContainsKey(x.Key) && !IsPartOfAnyPhraseInDictionary(x.Key) && !string.IsNullOrEmpty(x.Formatted) && x.Formatted.Contains(" ") && !IsChineseNumberSequence(x.Key)
				orderby x.Key.Length, x.Key
				select x.Key + "=" + x.Formatted).ToList();
			HashSet<string> hashSet = new HashSet<string>(list.Select((string t) => t.Split(new char[1] { '=' })[0]));
			foreach (Match item in regexExtractPhrases.Matches(textContent))
			{
				if (!item.Success)
				{
					continue;
				}
				string text = item.Value?.Trim();
				if (string.IsNullOrWhiteSpace(text))
				{
					continue;
				}
				string text2 = global::TranslatorEngine.TranslatorEngine.StandardizeInput(text, addLineBreaks: false);
				if (!hashSet.Contains(text2))
				{
					string text3 = BuildFormattedHanViet(text2);
					if (!string.IsNullOrEmpty(text3))
					{
						list.Add(text2 + "=" + text3);
						hashSet.Add(text2);
					}
				}
			}
			return (from x in list.Select(delegate(string item)
				{
					string value = item.Split(new char[1] { '=' })[0];
					int index = textContent.IndexOf(value, StringComparison.Ordinal);
					return new
					{
						Item = item,
						Index = index
					};
				})
				orderby (x.Index >= 0) ? x.Index : int.MaxValue
				select x.Item).ToList();
		}

		private static bool IsChineseNumberSequence(string text)
		{
			if (text.Length < 3)
			{
				return false;
			}
			return text.All((char c) => Enumerable.Contains("零一二三四五六七八九十百千万亿两〇", c));
		}

		public string ProcessAndCombineResults(string textContent, string apiResult, List<string> locOffLines)
		{
			var list = (from x in LocRacChinesePanel(apiResult, textContent).Split(new string[2] { "\r\n", "\n" }, StringSplitOptions.RemoveEmptyEntries).Select(delegate(string line)
				{
					string text2 = line.Split(new char[1] { '=' })[0].Trim();
					int index = textContent.IndexOf(text2, StringComparison.Ordinal);
					return new
					{
						Line = line,
						Key = text2,
						Index = index
					};
				})
				where x.Index >= 0
				orderby x.Index
				select x).ToList();
			List<string> values = list.Select(x => x.Line).ToList();
			HashSet<string> existedKeys = new HashSet<string>(list.Select(x => x.Key));
			List<string> values2 = locOffLines.Where(delegate(string line)
			{
				string[] array = line.Split(new char[1] { '=' });
				return array.Length == 2 && !existedKeys.Contains(array[0].Trim());
			}).ToList();
			string text = string.Join("\n", values2);
			return "========= Lọc API (MTC) =========\n" + string.Join("\n", values) + "\n========= Lọc Nội Bộ (QT) =========\n" + text;
		}

		private HashSet<string> FilterPhrasesAndNames(HashSet<string> phrases, string textContent)
		{
			HashSet<string> hashSet = FilterUnnecessaryItems(phrases, textContent);
			if (global::TranslatorEngine.TranslatorEngine.onlyNamePhuDictionary.Count <= 0)
			{
				return hashSet;
			}
			return FilterUnnecessaryItems(hashSet, textContent);
		}

		private HashSet<string> FilterUnnecessaryItems(HashSet<string> phrases, string textContent)
		{
			HashSet<string> hashSet = new HashSet<string>(phrases);
			HashSet<string> hashSet2 = new HashSet<string>();
			foreach (string phrase in phrases)
			{
				foreach (string key in global::TranslatorEngine.TranslatorEngine.onlyNamePhuDictionary.Keys)
				{
					if (key == phrase || !phrase.StartsWith(key))
					{
						continue;
					}
					string text = key.Substring(key.Length - 1);
					if (phrase.Substring(0, 1) == text)
					{
						string value = key + phrase.Substring(1);
						if (textContent.Contains(value))
						{
							hashSet2.Add(phrase);
							break;
						}
					}
				}
			}
			hashSet.ExceptWith(hashSet2);
			return hashSet;
		}

		public static bool IsPartOfAnyPhraseInDictionary(string locName)
		{
			if (locName.Length <= 1)
			{
				return false;
			}
			foreach (string key in global::TranslatorEngine.TranslatorEngine.onlyNamePhuDictionary.Keys)
			{
				if (key.Length > 1)
				{
					if (key.StartsWith(locName) || locName.StartsWith(key))
					{
						return true;
					}
					if (IsPhraseInBracketsMatch(locName, key) || IsPhraseInBracketsMatch(key, locName))
					{
						return true;
					}
					if (locName.Contains(key) && !MatchAnyStartEnd(locName, global::TranslatorEngine.TranslatorEngine.hoNguoiDictionary.Keys, isPrefix: true))
					{
						return true;
					}
				}
			}
			return false;
		}

		private static bool IsPhraseInBracketsMatch(string textWithBrackets, string textToCheck)
		{
			int num = textWithBrackets.IndexOf('《');
			int num2 = textWithBrackets.IndexOf('》');
			if (num != -1 && num2 > num)
			{
				string value = textWithBrackets.Substring(num + 1, num2 - num - 1).Trim();
				return textToCheck.Contains(value);
			}
			return false;
		}

		private HashSet<string> FilterUnnecessaryPhrasesOptimized(HashSet<string> phrases)
		{
			Dictionary<string, List<string>> dictionary = new Dictionary<string, List<string>>();
			foreach (string phrase in phrases)
			{
				if (phrase.Length >= 2)
				{
					string key = phrase.Substring(0, 2);
					if (!dictionary.ContainsKey(key))
					{
						dictionary[key] = new List<string>();
					}
					dictionary[key].Add(phrase);
				}
			}
			HashSet<string> hashSet = new HashSet<string>();
			foreach (List<string> value in dictionary.Values)
			{
				string item = value.OrderBy((string p) => p.Length).First();
				hashSet.Add(item);
			}
			return hashSet;
		}

		private bool ContainsNumber(string str)
		{
			return str.Any(char.IsDigit);
		}

		public static string BuildFormattedHanViet(string key, bool checkVietPhrase = true)
		{
			if (string.IsNullOrWhiteSpace(key))
			{
				return "";
			}
			if (key.Length > 2 && MatchAnyStartEnd(key, global::TranslatorEngine.TranslatorEngine.danhTuDictionary.Keys, isPrefix: false))
			{
				return TitleCaseWithDictionary(key);
			}
			if (checkVietPhrase && global::TranslatorEngine.TranslatorEngine.vietPhraseOneMeaningDictionary.TryGetValue(key, out var value))
			{
				if (string.IsNullOrWhiteSpace(value))
				{
					return TitleCaseChinese(key);
				}
				return value.Trim();
			}
			return TitleCaseChinese(key);
		}

		public static bool MatchAnyStartEnd(string text, IEnumerable<string> keys, bool isPrefix)
		{
			foreach (string key in keys)
			{
				if (text == key)
				{
					continue;
				}
				if (isPrefix)
				{
					if (text.StartsWith(key))
					{
						return true;
					}
				}
				else if (text.EndsWith(key))
				{
					return true;
				}
			}
			return false;
		}

		public static string TitleCaseChinese(string chineseText)
		{
			CharRange[] chineseHanVietMappingArray;
			string str = global::TranslatorEngine.TranslatorEngine.ChineseToHanViet(chineseText, out chineseHanVietMappingArray).Trim();
			return CultureInfo.CurrentCulture.TextInfo.ToTitleCase(str);
		}

		public static string TitleCaseWithDictionary(string key)
		{
			if (global::TranslatorEngine.TranslatorEngine.danhTuDictionary.TryGetValue(key, out var value))
			{
				return value.Replace("{0}", "").Trim();
			}
			foreach (KeyValuePair<string, string> item in global::TranslatorEngine.TranslatorEngine.danhTuDictionary.OrderByDescending((KeyValuePair<string, string> x) => x.Key.Length))
			{
				if (!string.IsNullOrEmpty(item.Key) && key.EndsWith(item.Key))
				{
					string value2 = item.Value;
					string text = key.Substring(0, key.Length - item.Key.Length).Trim();
					string value3;
					string text2 = ((text.Length <= 2) ? TitleCaseChinese(text).Trim() : ((!global::TranslatorEngine.TranslatorEngine.vietPhraseOneMeaningDictionary.TryGetValue(text, out value3)) ? TitleCaseChinese(text).Trim() : (IsTitleCase(value3) ? value3 : CultureInfo.CurrentCulture.TextInfo.ToTitleCase(value3))));
					if (value2.Contains("{0}"))
					{
						return value2.Replace("{0}", text2);
					}
					return text2 + " " + value2;
				}
			}
			return CultureInfo.CurrentCulture.TextInfo.ToTitleCase(key);
		}

		private static bool IsTitleCase(string input)
		{
			if (string.IsNullOrWhiteSpace(input))
			{
				return false;
			}
			string[] array = input.Split(new char[1] { ' ' }, StringSplitOptions.RemoveEmptyEntries);
			for (int i = 0; i < array.Length; i++)
			{
				string text = new string(array[i].Where(char.IsLetter).ToArray());
				if (!string.IsNullOrEmpty(text) && !char.IsUpper(text[0]))
				{
					return false;
				}
			}
			return true;
		}

		public string LocRacChinesePanel(string result, string textContent)
		{
			if (string.IsNullOrEmpty(result))
			{
				return string.Empty;
			}
			Dictionary<string, int> occurrences = new Dictionary<string, int>();
			string[] array = result.Split(new string[2] { "\r\n", "\n" }, StringSplitOptions.None);
			List<string> list = new List<string>();
			Regex regex = new Regex("[\\d\"'\t]|[。，！？；：、——\\\\/@\\.]", RegexOptions.Compiled);
			HashSet<string> phrases = new HashSet<string>(array.Where((string line) => !string.IsNullOrWhiteSpace(line)));
			bool flag = source.Length == 0;
			HashSet<string> hashSet = FilterPhrasesAndNames(phrases, textContent);
			HashSet<string> keysToRemove = GetKeysToRemove(hashSet);
			foreach (string key2 in hashSet)
			{
				bool num = global::TranslatorEngine.TranslatorEngine.onlyNameDictionary.ContainsKey(key2);
				bool flag2 = regex.IsMatch(key2);
				bool flag3 = IsPartOfAnyPhraseInDictionary(key2);
				bool flag4 = key2.Contains("……");
				if (!num && !flag2 && (flag || !source.Any((string s) => key2.Contains(s))) && !flag3 && !keysToRemove.Contains(key2) && !flag4)
				{
					list.Add(key2);
				}
			}
			Dictionary<string, int> keyFrequency = hashSet.ToDictionary((string k) => k, (string k) => (!occurrences.ContainsKey(k)) ? Regex.Matches(result, Regex.Escape(k)).Count : occurrences[k]);
			List<string> list2 = new List<string>();
			foreach (string item in from key in list
				let freq = keyFrequency.ContainsKey(key) ? keyFrequency[key] : 0
				orderby freq descending, key.Length, key
				select key)
			{
				string input = BuildFormattedHanViet(item);
				if (Enumerable.Contains(input, '·'))
				{
					input = AddSpacesToVietnamese(input);
				}
				input = Regex.Replace(input, "\\s+", " ");
				if (!item.Equals(input, StringComparison.OrdinalIgnoreCase) && !string.IsNullOrEmpty(input))
				{
					string text = AddSpacesToChinese(item);
					if (IsRepeatedTwiceOrMore(item, textContent, occurrences))
					{
						list2.Add(text + "=" + input);
					}
				}
			}
			return string.Join(Environment.NewLine, list2);
		}

		private HashSet<string> GetKeysToRemove(HashSet<string> keys)
		{
			HashSet<string> hashSet = new HashSet<string>();
			foreach (string key in keys)
			{
				foreach (string key2 in keys)
				{
					if (!(key == key2) && key2.Length == key.Length + 1 && key2.StartsWith(key))
					{
						hashSet.Add(key);
						break;
					}
				}
			}
			return hashSet;
		}

		private string AddSpacesToChinese(string input)
		{
			input = Regex.Replace(input, "(?<=\\p{IsCJKUnifiedIdeographs})(?=·)|(?<=·)(?=\\p{IsCJKUnifiedIdeographs})", " ");
			input = Regex.Replace(input, "\\.\\.\\.", ". . .");
			input = Regex.Replace(input, "(?<=\\p{IsCJKUnifiedIdeographs})(?=[a-zA-Z0-9+\\-])|(?<=[a-zA-Z0-9+\\-])(?=\\p{IsCJKUnifiedIdeographs})", " ");
			return input;
		}

		private string AddSpacesToVietnamese(string input)
		{
			return Regex.Replace(input, "(?<=\\p{L})(?=·)|(?<=·)(?=\\p{L})", " ");
		}

		private bool IsRepeatedTwiceOrMore(string item, string textContent, Dictionary<string, int> occurrences)
		{
			if (occurrences.TryGetValue(item, out var value))
			{
				return value >= 2;
			}
			int num = (occurrences[item] = Regex.Matches(textContent, Regex.Escape(item)).Count);
			if (num == 0)
			{
				return false;
			}
			if (item.Contains("《") && item.Contains("》"))
			{
				return true;
			}
			if (num >= 2)
			{
				return true;
			}
			if (MatchAnyStartEnd(item, global::TranslatorEngine.TranslatorEngine.hoNguoiDictionary.Keys, isPrefix: true))
			{
				return true;
			}
			if (MatchAnyStartEnd(item, global::TranslatorEngine.TranslatorEngine.danhTuDictionary.Keys, isPrefix: false))
			{
				return true;
			}
			return false;
		}

		public void HandleValueLogic(string value, StringBuilder stringBuilder, HashSet<string> uniqueLines)
		{
			if (string.IsNullOrEmpty(value))
			{
				return;
			}
			foreach (int item in FindChineseCodePoints(value))
			{
				string text = (char)item + "=thiếu phiên âm";
				if (uniqueLines.Add(text))
				{
					stringBuilder.AppendLine(text);
				}
			}
		}

		private static HashSet<int> FindChineseCodePoints(string text)
		{
			HashSet<int> hashSet = new HashSet<int>();
			for (int i = 0; i < text.Length; i++)
			{
				int num = char.ConvertToUtf32(text, i);
				if (IsChineseCodePoint(num))
				{
					hashSet.Add(num);
				}
				if (char.IsHighSurrogate(text[i]))
				{
					i++;
				}
			}
			return hashSet;
		}

		private static bool IsChineseCodePoint(int code)
		{
			if ((code < 13312 || code > 19903) && (code < 19968 || code > 40959))
			{
				if (code >= 131072)
				{
					return code <= 173791;
				}
				return false;
			}
			return true;
		}

		private void ValidateAndMergeTerms(HashSet<string> validTerms, List<string> segments)
		{
			HashSet<string> hashSet = new HashSet<string>();
			HashSet<string> hashSet2 = new HashSet<string>();
			validTerms.RemoveWhere((string term) => string.IsNullOrWhiteSpace(term));
			foreach (string validTerm in validTerms)
			{
				if (validTerm.Length == 2 && !global::TranslatorEngine.TranslatorEngine.onlyVietPhraseDictionary.ContainsKey(validTerm))
				{
					ProcessTwoCharacterTerm(validTerm, segments, hashSet, hashSet2);
				}
				else if (validTerm.Length == 4)
				{
					ProcessFourCharacterTerm(validTerm, hashSet, hashSet2);
				}
			}
			validTerms.ExceptWith(hashSet);
			validTerms.UnionWith(hashSet2);
		}

		private void ProcessTwoCharacterTerm(string term, List<string> segments, HashSet<string> termsToRemove, HashSet<string> termsToAdd)
		{
			bool flag = false;
			bool flag2 = MatchAnyStartEnd(term, global::TranslatorEngine.TranslatorEngine.hoNguoiDictionary.Keys, isPrefix: true);
			for (int i = 0; i < segments.Count - 1; i++)
			{
				if (segments[i] == term)
				{
					string text = segments[i + 1];
					if (global::TranslatorEngine.TranslatorEngine.danhTuDictionary.ContainsKey(text))
					{
						string item = term + text;
						termsToAdd.Add(item);
						flag = true;
					}
				}
			}
			if (!flag && !flag2)
			{
				termsToRemove.Add(term);
			}
			else
			{
				termsToAdd.Add(term);
			}
		}

		private void ProcessFourCharacterTerm(string term, HashSet<string> termsToRemove, HashSet<string> termsToAdd)
		{
			string key = term.Substring(0, 2);
			string key2 = term.Substring(2, 2);
			if (!global::TranslatorEngine.TranslatorEngine.danhTuDictionary.ContainsKey(key2) || (global::TranslatorEngine.TranslatorEngine.vietPhraseDictionary.ContainsKey(key) && global::TranslatorEngine.TranslatorEngine.vietPhraseDictionary.ContainsKey(key2)))
			{
				termsToRemove.Add(term);
			}
			else
			{
				termsToAdd.Add(term);
			}
		}
	}
	public class MainForm : Form

// ---- MainForm.GetThresholdBasedOnWordCount ----
		private int GetThresholdBasedOnWordCount(int numWords)
		{
			if (numWords < 50000)
			{
				return 1;
			}
			if (numWords < 100000)
			{
				return 2;
			}
			if (numWords < 200000)
			{
				return 3;
			}
			return 4;
		}

