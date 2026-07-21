using System;
using System.Collections.Generic;
using System.Data;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Net;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Runtime.Versioning;
using System.Security;
using System.Security.Permissions;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.CodeAnalysis;
using Microsoft.VisualBasic;
using org.mozilla.intl.chardet;

[assembly: CompilationRelaxations(8)]
[assembly: RuntimeCompatibility(WrapNonExceptionThrows = true)]
[assembly: Debuggable(DebuggableAttribute.DebuggingModes.IgnoreSymbolStoreSequencePoints)]
[assembly: TargetFramework(".NETFramework,Version=v4.5", FrameworkDisplayName = ".NET Framework 4.5")]
[assembly: SecurityPermission(SecurityAction.RequestMinimum, SkipVerification = true)]
[assembly: AssemblyVersion("19.11.92.0")]
[module: UnverifiableCode]
[module: RefSafetyRules(11)]
namespace Microsoft.CodeAnalysis
{
	[CompilerGenerated]
	[Microsoft.CodeAnalysis.Embedded]
	internal sealed class EmbeddedAttribute : Attribute
	{
	}
}
namespace System.Runtime.CompilerServices
{
	[CompilerGenerated]
	[Microsoft.CodeAnalysis.Embedded]
	[AttributeUsage(AttributeTargets.Module, AllowMultiple = false, Inherited = false)]
	internal sealed class RefSafetyRulesAttribute : Attribute
	{
		public readonly int Version;

		public RefSafetyRulesAttribute(int P_0)
		{
			Version = P_0;
		}
	}
}
namespace TranslatorEngine
{
	public class ApplicationLog
	{
		public static void Log(string applicationPath, string application, Exception exception)
		{
			try
			{
				string text = Path.Combine(applicationPath, application + ".log");
				FileInfo fileInfo = new FileInfo(text);
				if (fileInfo.Exists && 1000000 < fileInfo.Length)
				{
					fileInfo.Delete();
				}
				string contents = string.Format("{0:G}: {1}\r\n", DateTime.Now, string.Concat(exception.Message, "\r\n", exception.GetType(), "\r\n", exception.StackTrace));
				File.AppendAllText(text, contents, Encoding.UTF8);
			}
			catch
			{
			}
		}
	}
	public class CharRange
	{
		private int startIndex;

		private int length;

		public int StartIndex
		{
			get
			{
				return startIndex;
			}
			set
			{
				startIndex = value;
			}
		}

		public int Length
		{
			get
			{
				return length;
			}
			set
			{
				length = value;
			}
		}

		public CharRange(int startIndex, int length)
		{
			this.startIndex = startIndex;
			this.length = length;
		}

		public bool IsInRange(int index)
		{
			if (startIndex <= index)
			{
				return index <= startIndex + length - 1;
			}
			return false;
		}

		public int GetEndIndex()
		{
			return startIndex + length - 1;
		}
	}
	public class CharsetDetector
	{
		public static string DetectedCharset;

		public static string DetectChineseCharset(string filePath)
		{
			DetectedCharset = "GB2312";
			nsDetector nsDetector = new nsDetector(3);
			Notifier aObserver = new Notifier();
			nsDetector.Init(aObserver);
			byte[] array = new byte[1024];
			int aLen = File.OpenRead(filePath).Read(array, 0, array.Length);
			bool num = nsDetector.isAscii(array, aLen);
			if (!num)
			{
				nsDetector.DoIt(array, aLen, oDontFeedMe: false);
			}
			nsDetector.DataEnd();
			if (num)
			{
				DetectedCharset = "ASCII";
			}
			if (File.ReadAllText(filePath).Contains("CONTENT=\"text/html; charset=gb2312\""))
			{
				DetectedCharset = "GB2312";
			}
			return DetectedCharset;
		}
	}
	public class DictionaryConfigurationHelper
	{
		private static string directoryPath = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location);

		private static readonly object configFileLock = new object();

		public static string GetNamesDictionaryPath()
		{
			return GetDictionaryPathByKey("Names");
		}

		public static string GetNamesDictionaryHistoryPath()
		{
			return Path.Combine(Path.GetDirectoryName(GetNamesDictionaryPath()), "NamesHistory" + Path.GetExtension(GetNamesDictionaryPath()));
		}

		public static string GetNamesPhuDictionaryPath()
		{
			return GetDictionaryPathByKey("NamesPhu");
		}

		public static string GetNamesPhuDictionaryHistoryPath()
		{
			return Path.Combine(Path.GetDirectoryName(GetNamesPhuDictionaryPath()), "Names2History" + Path.GetExtension(GetNamesPhuDictionaryPath()));
		}

		public static string GetVietPhraseDictionaryPath()
		{
			return GetDictionaryPathByKey("VietPhrase");
		}

		public static string GetVietPhraseDictionaryHistoryPath()
		{
			return Path.Combine(Path.GetDirectoryName(GetVietPhraseDictionaryPath()), "VietPhraseHistory" + Path.GetExtension(GetVietPhraseDictionaryPath()));
		}

		public static string GetChinesePhienAmWordsDictionaryPath()
		{
			return GetDictionaryPathByKey("ChinesePhienAmWords");
		}

		public static string GetDanhTuDictionaryPath()
		{
			return GetDictionaryPathByKey("DanhTu");
		}

		public static string GetHoNguoiDictionaryPath()
		{
			return GetDictionaryPathByKey("HoNguoi");
		}

		public static string GetHauTuDictionaryPath()
		{
			return GetDictionaryPathByKey("HauTu");
		}

		public static string GetChinesePhienAmWordsDictionaryHistoryPath()
		{
			return Path.Combine(Path.GetDirectoryName(GetChinesePhienAmWordsDictionaryPath()), "ChinesePhienAmWordsHistory" + Path.GetExtension(GetChinesePhienAmWordsDictionaryPath()));
		}

		public static string GetLacVietDictionaryPath()
		{
			return GetDictionaryPathByKey("LacViet");
		}

		public static string GetIgnoredChinesePhraseListPath()
		{
			return GetDictionaryPathByKey("IgnoredChinesePhrases");
		}

		public static string GetLuatNhanDictionaryPath()
		{
			return GetDictionaryPathByKey("LuatNhan");
		}

		public static string GetPronounsDictionaryPath()
		{
			return GetDictionaryPathByKey("Pronouns");
		}

		private static string GetDictionaryPathByKey(string dictionaryKey)
		{
			lock (configFileLock)
			{
				string path = Path.Combine(directoryPath, "Dictionaries.config");
				string[] array = null;
				for (int i = 0; i < 3; i++)
				{
					try
					{
						array = File.ReadAllLines(path);
					}
					catch (IOException)
					{
						Thread.Sleep(100);
						continue;
					}
					break;
				}
				if (array == null)
				{
					throw new IOException("Unable to read Dictionaries.config after 3 attempts.");
				}
				string text = string.Empty;
				string[] array2 = array;
				foreach (string text2 in array2)
				{
					if (!string.IsNullOrEmpty(text2) && !text2.StartsWith("#") && text2.StartsWith(dictionaryKey + "="))
					{
						text = text2.Split(new char[1] { '=' })[1];
						break;
					}
				}
				if (!Path.IsPathRooted(text))
				{
					text = Path.Combine(directoryPath, text);
				}
				if (!File.Exists(text) && text.EndsWith(".txt", StringComparison.OrdinalIgnoreCase) && (text.Contains("Names2\\") || text.Contains("VietPhrase\\")))
				{
					string directoryName = Path.GetDirectoryName(text);
					if (!Directory.Exists(directoryName))
					{
						Directory.CreateDirectory(directoryName);
					}
					using (File.Create(text))
					{
					}
					UpdateConfigFile(dictionaryKey, text);
					Thread.Sleep(50);
				}
				return text;
			}
		}

		private static void UpdateConfigFile(string key, string newPath)
		{
			lock (configFileLock)
			{
				string path = Path.Combine(directoryPath, "Dictionaries.config");
				string text = newPath.Replace(directoryPath + "\\", "").Replace(directoryPath + "/", "");
				string[] array = new string[0];
				for (int i = 0; i < 3; i++)
				{
					try
					{
						if (File.Exists(path))
						{
							array = File.ReadAllLines(path);
						}
					}
					catch (IOException)
					{
						Thread.Sleep(100);
						if (i == 2)
						{
							throw;
						}
						continue;
					}
					break;
				}
				bool flag = false;
				List<string> list = new List<string>();
				string[] array2 = array;
				foreach (string text2 in array2)
				{
					if (!string.IsNullOrWhiteSpace(text2) && !text2.StartsWith("#") && text2.StartsWith(key + "="))
					{
						list.Add(key + "=" + text);
						flag = true;
					}
					else
					{
						list.Add(text2);
					}
				}
				if (!flag)
				{
					list.Add(key + "=" + text);
				}
				for (int k = 0; k < 3; k++)
				{
					try
					{
						File.WriteAllLines(path, list);
						break;
					}
					catch (IOException)
					{
						Thread.Sleep(100);
						if (k == 2)
						{
							throw;
						}
					}
				}
			}
		}
	}
	public class DictionaryEventArgs : EventArgs
	{
		public int VietPhraseCount { get; set; }

		public int NameCount { get; set; }

		public int NamePhuCount { get; set; }

		public DictionaryEventArgs(int VietPhraseCount, int NameCount, int NamePhuCount)
		{
			this.VietPhraseCount = VietPhraseCount;
			this.NameCount = NameCount;
			this.NamePhuCount = NamePhuCount;
		}
	}
	public class HtmlParser
	{
		private static bool dirty = true;

		private static string directoryPath = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location);

		private static string[] titleTags;

		private static string[] contentTags;

		private static string[] removedTags;

		public static string GetChineseContent(string htmlContent, bool needMarkChapterHeaders)
		{
			LoadConfiguration();
			StringBuilder stringBuilder = new StringBuilder();
			string[] array = titleTags;
			foreach (string text in array)
			{
				if (!string.IsNullOrEmpty(text) && !text.StartsWith("#") && htmlContent.ToLower().Contains(text.ToLower()))
				{
					string text2 = htmlContent.Substring(htmlContent.ToLower().IndexOf(text.ToLower()) + text.Length);
					string text3 = text.Substring(text.LastIndexOf('<') + 1);
					string text4 = text3.Substring(0, text3.IndexOfAny(new char[2] { ' ', '>' }));
					if (text2.ToLower().Contains("</" + text4.ToLower() + ">"))
					{
						stringBuilder.AppendLine((needMarkChapterHeaders ? "$CHAPTER_HEADER$. " : "") + text2.Substring(0, text2.ToLower().IndexOf("</" + text4.ToLower() + ">")).TrimStart(" \u3000\t".ToCharArray()));
						break;
					}
				}
			}
			array = contentTags;
			foreach (string text5 in array)
			{
				if (string.IsNullOrEmpty(text5) || text5.StartsWith("#") || !htmlContent.ToLower().Contains(text5.ToLower()))
				{
					continue;
				}
				string text6 = htmlContent.Substring(htmlContent.ToLower().IndexOf(text5.ToLower()) + text5.Length);
				if (text5 != "<!--bodybegin-->")
				{
					string text7 = text5.Substring(text5.LastIndexOf('<') + 1);
					string text8 = text7.Substring(0, text7.IndexOfAny(new char[2] { ' ', '>' }));
					if (text6.ToLower().Contains("</" + text8.ToLower().TrimStart(new char[1] { '/' }) + ">"))
					{
						stringBuilder.AppendLine(text6.Substring(0, text6.ToLower().IndexOf("</" + text8.ToLower().TrimStart(new char[1] { '/' }) + ">")));
						break;
					}
				}
				else
				{
					string text9 = "<!--bodyend-->";
					if (text6.Contains(text9))
					{
						stringBuilder.AppendLine(text6.Substring(0, text6.ToLower().IndexOf(text9.ToLower())));
					}
				}
			}
			string text10 = stringBuilder.ToString();
			array = removedTags;
			foreach (string text11 in array)
			{
				if (!string.IsNullOrEmpty(text11) && !text11.StartsWith("#"))
				{
					text10 = text10.Replace(text11, string.Empty);
				}
			}
			return Regex.Replace(text10.Replace("<p>", "\n").Replace("</p>", "\n").Replace("<br>", "\n")
				.Replace("<br/>", "\n")
				.Replace("<br />", "\n")
				.Replace("<BR>", "\n")
				.Replace("<BR/>", "\n")
				.Replace("<BR />", "\n")
				.Replace("&nbsp;", "")
				.Replace("&lt;", "")
				.Replace("&gt;", ""), "<(.|\\n)*?>", string.Empty);
		}

		private static void LoadConfiguration()
		{
			if (dirty)
			{
				titleTags = File.ReadAllLines(Path.Combine(directoryPath, "HtmlChapterTitleTags.config"));
				contentTags = File.ReadAllLines(Path.Combine(directoryPath, "HtmlChapterContentTags.config"));
				removedTags = File.ReadAllLines(Path.Combine(directoryPath, "HtmlRemovedTags.config"));
				dirty = false;
			}
		}
	}
	public class Notifier : nsICharsetDetectionObserver
	{
		public void Notify(string charset)
		{
			CharsetDetector.DetectedCharset = charset;
		}
	}
	public class TransLuatNhan
	{
		private static readonly Regex NumberPatternRegex = new Regex("(?:(?:\\d+(?:[.,]\\d+)?|[零一二三四五六七八九十百千万亿两〇]+)[ \\t]*)+", RegexOptions.Compiled);

		private static readonly Regex MixedNumberRegex = new Regex("^(\\d+)\\s*([万亿])\\s*(\\d)$", RegexOptions.Compiled);

		private static readonly Regex ComplexRangeRegex = new Regex("^([十百千]+?)([一二三四五六七八九])([一二三四五六七八九])([万亿])$", RegexOptions.Compiled);

		private static readonly Regex SimpleRangeRegex = new Regex("^([一二三四五六七八九两])([一二三四五六七八九])([十百千万亿])$", RegexOptions.Compiled);

		private static readonly Regex SimpleRangeWithUnitRegex = new Regex("^([一二三四五六七八九两])([一二三四五六七八九])([十百千])([万亿])$", RegexOptions.Compiled);

		private static readonly Regex SuffixRangeRegex = new Regex("^(.*[万亿])([一二三四五六七八九])([一二三四五六七八九])$", RegexOptions.Compiled);

		private static readonly CultureInfo EnUS = new CultureInfo("en-US");

		private static readonly Dictionary<char, char> LatinToChineseMap = new Dictionary<char, char>
		{
			{ '0', '零' },
			{ '1', '一' },
			{ '2', '二' },
			{ '3', '三' },
			{ '4', '四' },
			{ '5', '五' },
			{ '6', '六' },
			{ '7', '七' },
			{ '8', '八' },
			{ '9', '九' }
		};

		public static readonly Dictionary<char, int> chineseNumberMap = new Dictionary<char, int>
		{
			['零'] = 0,
			['〇'] = 0,
			['一'] = 1,
			['二'] = 2,
			['两'] = 2,
			['三'] = 3,
			['四'] = 4,
			['五'] = 5,
			['六'] = 6,
			['七'] = 7,
			['八'] = 8,
			['九'] = 9
		};

		public static readonly Dictionary<char, int> chineseUnitMapNone = new Dictionary<char, int>
		{
			['十'] = 10,
			['百'] = 100,
			['千'] = 1000,
			['万'] = 10000,
			['亿'] = 100000000
		};

		public static bool TryTranslateConsecutiveChineseNumbers(string chineseNumber, out string result)
		{
			result = null;
			if (string.IsNullOrEmpty(chineseNumber) || !chineseNumber.All((char c) => chineseNumberMap.ContainsKey(c)))
			{
				return false;
			}
			if (chineseNumber.Length >= 3)
			{
				StringBuilder stringBuilder = new StringBuilder();
				foreach (char key in chineseNumber)
				{
					if (chineseNumberMap.TryGetValue(key, out var value))
					{
						stringBuilder.Append(value);
					}
				}
				result = stringBuilder.ToString();
				return true;
			}
			return false;
		}

		public static string TranslateSLuongRule(string chineseText)
		{
			if (string.IsNullOrEmpty(chineseText) || !chineseText.EndsWith("两"))
			{
				return null;
			}
			string text = chineseText.Substring(0, chineseText.Length - 1);
			if (string.IsNullOrEmpty(text))
			{
				return null;
			}
			try
			{
				return NumberToVietnameseText(ConvertChineseNumberToLong(text)) + " lượng";
			}
			catch (Exception)
			{
				return null;
			}
		}

		public static string FindLongestNumber(string text, int startIndex, out int matchedLength, out TranslatorEngine.NumberType numberType)
		{
			matchedLength = 0;
			numberType = TranslatorEngine.NumberType.NotANumber;
			Match match = NumberPatternRegex.Match(text, startIndex);
			if (match.Success && match.Index == startIndex)
			{
				string value = match.Value;
				string text2 = value.TrimEnd(new char[0]);
				matchedLength = match.Length - (value.Length - text2.Length);
				if (text2.Length == 1)
				{
					char value2 = text2[0];
					if (Enumerable.Contains("百千万亿", value2))
					{
						return null;
					}
				}
				bool flag = false;
				bool flag2 = false;
				StringBuilder stringBuilder = new StringBuilder(text2.Length);
				string text3 = text2;
				foreach (char c2 in text3)
				{
					if (char.IsDigit(c2))
					{
						flag = true;
						if (LatinToChineseMap.TryGetValue(c2, out var value3))
						{
							stringBuilder.Append(value3);
						}
					}
					else if (Enumerable.Contains("零一二三四五六七八九十百千万亿两〇", c2))
					{
						flag2 = true;
						stringBuilder.Append(c2);
					}
				}
				if (flag && flag2)
				{
					numberType = TranslatorEngine.NumberType.Mixed;
					if (text2.Any((char c) => Enumerable.Contains("零一二三四五六七八九两〇", c)))
					{
						return stringBuilder.ToString().Replace(" ", "");
					}
					return text2.Trim();
				}
				if (flag2)
				{
					numberType = TranslatorEngine.NumberType.PureChinese;
				}
				else if (flag)
				{
					if (Enumerable.Contains(text2, '.') || Enumerable.Contains(text2, ',') || text2.StartsWith("0"))
					{
						numberType = TranslatorEngine.NumberType.LatinDecimal;
					}
					else
					{
						numberType = TranslatorEngine.NumberType.PureLatin;
					}
				}
				return text2.Trim();
			}
			return null;
		}

		public static string NumberToVietnameseText(long number)
		{
			if (number == 0L)
			{
				return "0";
			}
			if (number > -10000 && number < 10000)
			{
				return number.ToString("N0", EnUS);
			}
			List<string> list = new List<string>();
			long num = number / 100000000;
			if (num > 0)
			{
				list.Add(NumberToVietnameseText(num) + " ức");
				number %= 100000000;
			}
			long num2 = number / 10000;
			if (num2 > 0)
			{
				list.Add(num2.ToString("N0", EnUS) + " vạn");
				number %= 10000;
			}
			if (number > 0)
			{
				list.Add(number.ToString("N0", EnUS));
			}
			return string.Join(" ", list).Trim();
		}

		public static bool TryConvertPostfixedRangeNumber(string chineseNumber, out string result)
		{
			result = string.Empty;
			int length = chineseNumber.Length;
			if (length < 3)
			{
				return false;
			}
			if (chineseNumberMap.TryGetValue(chineseNumber[length - 2], out var value) && chineseNumberMap.TryGetValue(chineseNumber[length - 1], out var value2))
			{
				if (value == 0)
				{
					return false;
				}
				string text = chineseNumber.Substring(0, length - 2);
				if (string.IsNullOrEmpty(text))
				{
					return false;
				}
				long num = ConvertChineseNumberToLong(text);
				if (num > 0 && num % 10 == 0L)
				{
					long num2 = num + value;
					long num3 = num + value2;
					result = $"{num2}-{num3}";
					return true;
				}
			}
			return false;
		}

		private static long ConvertUnderTenThousand(string chineseNumber)
		{
			if (string.IsNullOrEmpty(chineseNumber))
			{
				return 0L;
			}
			if (chineseNumber == "十")
			{
				return 10L;
			}
			if (chineseNumber.StartsWith("十"))
			{
				return 10 + ConvertUnderTenThousand(chineseNumber.Substring(1));
			}
			long num = 0L;
			long num2 = 0L;
			foreach (char key in chineseNumber)
			{
				int value2;
				if (chineseNumberMap.TryGetValue(key, out var value))
				{
					num2 = value;
				}
				else if (chineseUnitMapNone.TryGetValue(key, out value2))
				{
					num += ((num2 == 0L) ? 1 : num2) * value2;
					num2 = 0L;
				}
			}
			return num + num2;
		}

		public static long ConvertChineseNumberToLong(string chineseNumber)
		{
			if (string.IsNullOrEmpty(chineseNumber))
			{
				return 0L;
			}
			string text = chineseNumber.Trim();
			if (long.TryParse(text, out var result))
			{
				return result;
			}
			Match match = MixedNumberRegex.Match(text);
			if (match.Success)
			{
				long num = long.Parse(match.Groups[1].Value);
				char num2 = match.Groups[2].Value[0];
				long num3 = long.Parse(match.Groups[3].Value);
				long num4 = ((num2 == '万') ? 10000 : 100000000);
				return num * num4 + num3 * (num4 / 10);
			}
			int num5 = text.LastIndexOf('亿');
			if (num5 > -1)
			{
				string text2 = text.Substring(0, num5);
				string text3 = text.Substring(num5 + 1);
				long num6 = (string.IsNullOrEmpty(text2) ? 1 : ConvertChineseNumberToLong(text2)) * 100000000;
				if (!string.IsNullOrEmpty(text3) && text3.Length == 1 && chineseNumberMap.TryGetValue(text3[0], out var value) && !chineseUnitMapNone.ContainsKey(text3[0]))
				{
					return num6 + (long)value * 10000000L;
				}
				return num6 + ConvertChineseNumberToLong(text3);
			}
			int num7 = text.LastIndexOf('万');
			if (num7 > -1)
			{
				string text4 = text.Substring(0, num7);
				string text5 = text.Substring(num7 + 1);
				long num8 = (string.IsNullOrEmpty(text4) ? 1 : ConvertChineseNumberToLong(text4)) * 10000;
				if (!string.IsNullOrEmpty(text5) && text5.Length == 1 && chineseNumberMap.TryGetValue(text5[0], out var value2) && !chineseUnitMapNone.ContainsKey(text5[0]))
				{
					return num8 + (long)value2 * 1000L;
				}
				return num8 + ConvertChineseNumberToLong(text5);
			}
			if (text.Length >= 3 && !text.Any((char c) => chineseUnitMapNone.ContainsKey(c)) && text.All((char c) => chineseNumberMap.ContainsKey(c)))
			{
				StringBuilder stringBuilder = new StringBuilder();
				string text6 = text;
				foreach (char key in text6)
				{
					stringBuilder.Append(chineseNumberMap[key]);
				}
				if (long.TryParse(stringBuilder.ToString(), out var result2))
				{
					return result2;
				}
			}
			return ConvertUnderTenThousand(text);
		}

		public static bool TryConvertVietnameseRangeNumber(string chineseNumber, out string result)
		{
			result = string.Empty;
			if (string.IsNullOrEmpty(chineseNumber))
			{
				return false;
			}
			Match match = ComplexRangeRegex.Match(chineseNumber);
			if (match.Success)
			{
				string value = match.Groups[1].Value;
				char key = match.Groups[2].Value[0];
				char key2 = match.Groups[3].Value[0];
				char key3 = match.Groups[4].Value[0];
				long num = ConvertChineseNumberToLong(value);
				if (!chineseNumberMap.TryGetValue(key, out var value2) || !chineseNumberMap.TryGetValue(key2, out var value3) || !chineseUnitMapNone.TryGetValue(key3, out var value4))
				{
					return false;
				}
				if (value2 >= value3)
				{
					return false;
				}
				long num2 = 1L;
				if (value.EndsWith("千"))
				{
					num2 = 100L;
				}
				else if (value.EndsWith("百"))
				{
					num2 = 10L;
				}
				long num3 = num + value2 * num2;
				long num4 = num + value3 * num2;
				long number = num3 * value4;
				long number2 = num4 * value4;
				result = NumberToVietnameseText(number) + "-" + NumberToVietnameseText(number2);
				return true;
			}
			Match match2 = SimpleRangeWithUnitRegex.Match(chineseNumber);
			if (match2.Success)
			{
				char key4 = match2.Groups[1].Value[0];
				char key5 = match2.Groups[2].Value[0];
				char key6 = match2.Groups[3].Value[0];
				char c = match2.Groups[4].Value[0];
				if (!chineseNumberMap.TryGetValue(key4, out var value5) || !chineseNumberMap.TryGetValue(key5, out var value6) || !chineseUnitMapNone.TryGetValue(key6, out var value7))
				{
					return false;
				}
				if (value5 >= value6)
				{
					return false;
				}
				string arg = ((c == '万') ? "vạn" : "ức");
				long num5 = (long)value5 * (long)value7;
				long num6 = (long)value6 * (long)value7;
				result = $"{num5}-{num6} {arg}";
				return true;
			}
			Match match3 = SimpleRangeRegex.Match(chineseNumber);
			if (match3.Success)
			{
				char key7 = match3.Groups[1].Value[0];
				char key8 = match3.Groups[2].Value[0];
				char key9 = match3.Groups[3].Value[0];
				if (!chineseNumberMap.TryGetValue(key7, out var value8) || !chineseNumberMap.TryGetValue(key8, out var value9) || !chineseUnitMapNone.TryGetValue(key9, out var value10))
				{
					return false;
				}
				if (value8 >= value9)
				{
					return false;
				}
				switch (value10)
				{
				case 100000000:
					result = $"{value8}-{value9} ức";
					break;
				case 10000:
					result = $"{value8}-{value9} vạn";
					break;
				case 1000:
					result = $"{value8}-{value9} ngàn";
					break;
				default:
					result = $"{value8 * value10}-{value9 * value10}";
					break;
				}
				return true;
			}
			Match match4 = SuffixRangeRegex.Match(chineseNumber);
			if (match4.Success)
			{
				string value11 = match4.Groups[1].Value;
				char key10 = match4.Groups[2].Value[0];
				char key11 = match4.Groups[3].Value[0];
				long num7 = ConvertChineseNumberToLong(value11);
				if (!chineseNumberMap.TryGetValue(key10, out var value12) || !chineseNumberMap.TryGetValue(key11, out var value13))
				{
					return false;
				}
				if (value12 >= value13)
				{
					return false;
				}
				long num8 = 0L;
				if (value11.EndsWith("亿"))
				{
					num8 = 10000000L;
				}
				else if (value11.EndsWith("万"))
				{
					num8 = 1000L;
				}
				if (num8 > 0)
				{
					long number3 = num7 + value12 * num8;
					long number4 = num7 + value13 * num8;
					result = NumberToVietnameseText(number3) + "-" + NumberToVietnameseText(number4);
					return true;
				}
			}
			return false;
		}

		public static string ConvertChineseDecimalToString(string chineseNumber)
		{
			if (string.IsNullOrWhiteSpace(chineseNumber))
			{
				return string.Empty;
			}
			string text = chineseNumber.Trim();
			if (text.Contains("点"))
			{
				string[] array = text.Split(new char[1] { '点' });
				string chineseNumber2 = array[0];
				string text2 = ((array.Length > 1) ? array[1] : "");
				string text3 = ConvertChineseNumberToLong(chineseNumber2).ToString();
				StringBuilder stringBuilder = new StringBuilder();
				string text4 = text2;
				foreach (char c2 in text4)
				{
					if (chineseNumberMap.TryGetValue(c2, out var value))
					{
						stringBuilder.Append(value);
					}
					else
					{
						stringBuilder.Append(c2);
					}
				}
				return text3 + "." + stringBuilder.ToString();
			}
			if (text.Length >= 2 && text.All((char c) => chineseNumberMap.ContainsKey(c)) && !text.Any((char c) => chineseUnitMapNone.ContainsKey(c)))
			{
				StringBuilder stringBuilder2 = new StringBuilder();
				string text4 = text;
				foreach (char key in text4)
				{
					stringBuilder2.Append(chineseNumberMap[key]);
				}
				return stringBuilder2.ToString();
			}
			if (text.Length == 3 && text[2] == '十' && chineseNumberMap.TryGetValue(text[0], out var value2) && chineseNumberMap.TryGetValue(text[1], out var value3))
			{
				return $"{value2 * 10}-{value3 * 10}";
			}
			return ConvertChineseNumberToLong(text).ToString();
		}
	}
	public class TranslationResult
	{
		public string TranslatedText { get; set; }

		public CharRange[] ChinesePhraseRanges { get; set; }

		public CharRange[] VietPhraseRanges { get; set; }
	}
	public class TranslatorEngine
	{
		public enum NumberType
		{
			NotANumber,
			PureLatin,
			PureChinese,
			Mixed,
			LatinDecimal
		}

		public enum DCType
		{
			HauTu,
			DanhTu,
			HoNguoi,
			HanViet,
			Pronoun,
			LacViet
		}

		private class NumberInfo
		{
			public int StartIndex { get; set; }

			public int Length { get; set; }

			public string Text { get; set; }

			public NumberType Type { get; set; }
		}

		public static Dictionary<string, string> hanVietDictionary;

		public static Dictionary<string, string> danhTuDictionary;

		public static Dictionary<string, string> hoNguoiDictionary;

		public static Dictionary<string, string> hauTuDictionary;

		public static Dictionary<string, string> vietPhraseDictionary;

		private static Dictionary<string, string> lacVietDictionary;

		public static Dictionary<string, string> vietPhraseOneMeaningDictionary;

		public static Dictionary<string, string> onlyVietPhraseDictionary;

		public static Dictionary<string, string> onlyNameDictionary;

		private static Dictionary<string, string> onlyNameOneMeaningDictionary;

		public static Dictionary<string, string> onlyNameChinhDictionary;

		public static Dictionary<string, string> onlyNamePhuDictionary;

		public static Dictionary<string, string> luatNhanDictionary;

		private static Dictionary<string, string> pronounDictionary;

		private static DataSet onlyVietPhraseDictionaryHistoryDataSet;

		private static DataSet onlyNameDictionaryHistoryDataSet;

		private static DataSet onlyNamePhuDictionaryHistoryDataSet;

		private static DataSet hanVietDictionaryHistoryDataSet;

		private static List<string> ignoredChinesePhraseList;

		private static List<string> ignoredChinesePhraseForBrowserList;

		private static object lockObject;

		private static string NULL_STRING;

		private const int CN_scanRange = 20;

		public static string LastTranslatedWord_HanViet;

		public static string LastTranslatedWord_VietPhrase;

		public static string LastTranslatedWord_VietPhraseOneMeaning;

		private static StringBuilder resultHanViet;

		private static Dictionary<string, string> luatNhanNList;

		private static Dictionary<string, string> luatNhanSList;

		private static Dictionary<string, string> dictionaryN;

		private static Dictionary<string, Regex> luatNhanSCache;

		private static readonly Dictionary<string, Regex> luatNhanNCache;

		public static HashSet<string> hoHauTuCache;

		private static readonly HashSet<char> NumberChars;

		private const string baseNumberPattern = "(?:[零一二三四五六七八九十百千万亿两〇\\d]+)";

		private const string basePattern = "([零一二三四五六七八九十百千万亿两〇点\\d]+)";

		private const string numberPatternForCache = "((?:\\d+\\s*[万亿])|(?:\\d+)|(?:[零一二三四五六七八九十百千万亿两〇]+))";

		private static bool dictionaryDirty;

		private static bool isLoading;

		private static Dictionary<DCType, int> changeCounters;

		public event EventHandler<DictionaryEventArgs> DataDictionaryLoaded;

		public static void MarkCacheAsDirty(DCType cacheType)
		{
			lock (lockObject)
			{
				if (changeCounters.ContainsKey(cacheType))
				{
					changeCounters[cacheType]++;
				}
			}
		}

		public static string GetNameValueFromKey(string key, bool isNameChinh)
		{
			if (!(isNameChinh ? onlyNameChinhDictionary : onlyNamePhuDictionary).TryGetValue(key, out var value))
			{
				return null;
			}
			return value;
		}

		public static void DeleteKeyFromVietPhraseDictionary(string key, bool sorting)
		{
			vietPhraseDictionary.Remove(key);
			vietPhraseOneMeaningDictionary.Remove(key);
			onlyVietPhraseDictionary.Remove(key);
			if (sorting)
			{
				SaveDictionaryToFile(ref onlyVietPhraseDictionary, DictionaryConfigurationHelper.GetVietPhraseDictionaryPath());
			}
			else
			{
				SaveDictionaryToFileWithoutSorting(onlyVietPhraseDictionary, DictionaryConfigurationHelper.GetVietPhraseDictionaryPath());
			}
			writeVietPhraseHistoryLog(key, "Deleted");
		}

		public static void DeleteKeyFromNameDictionary(string key, bool sorting, bool isNameChinh)
		{
			vietPhraseDictionary.Remove(key);
			vietPhraseOneMeaningDictionary.Remove(key);
			onlyNameDictionary.Remove(key);
			onlyNameOneMeaningDictionary.Remove(key);
			Dictionary<string, string> dictionary = (isNameChinh ? onlyNameChinhDictionary : onlyNamePhuDictionary);
			if (dictionary.ContainsKey(key))
			{
				dictionary.Remove(key);
				if (sorting)
				{
					SaveDictionaryToFile(ref dictionary, isNameChinh ? DictionaryConfigurationHelper.GetNamesDictionaryPath() : DictionaryConfigurationHelper.GetNamesPhuDictionaryPath());
				}
				else
				{
					SaveDictionaryToFileWithoutSorting(dictionary, isNameChinh ? DictionaryConfigurationHelper.GetNamesDictionaryPath() : DictionaryConfigurationHelper.GetNamesPhuDictionaryPath());
				}
				writeNamesHistoryLog(key, "Deleted", isNameChinh);
			}
		}

		public static void UpdateVietPhraseDictionary(string key, string value, bool sorting)
		{
			string value2 = value.Split('/', '|')[0];
			vietPhraseDictionary[key] = value;
			vietPhraseOneMeaningDictionary[key] = value2;
			bool flag = onlyVietPhraseDictionary.ContainsKey(key);
			if (flag)
			{
				onlyVietPhraseDictionary[key] = value;
			}
			else if (sorting)
			{
				onlyVietPhraseDictionary.Add(key, value);
			}
			else
			{
				onlyVietPhraseDictionary = AddEntryToDictionaryWithoutSorting(onlyVietPhraseDictionary, key, value);
			}
			writeVietPhraseHistoryLog(key, flag ? "Updated" : "Added");
			if (sorting)
			{
				SaveDictionaryToFile(ref onlyVietPhraseDictionary, DictionaryConfigurationHelper.GetVietPhraseDictionaryPath());
			}
			else
			{
				SaveDictionaryToFileWithoutSorting(onlyVietPhraseDictionary, DictionaryConfigurationHelper.GetVietPhraseDictionaryPath());
			}
		}

		private static Dictionary<string, string> AddEntryToDictionaryWithoutSorting(Dictionary<string, string> dictionary, string key, string value)
		{
			Dictionary<string, string> dictionary2 = new Dictionary<string, string>();
			foreach (KeyValuePair<string, string> item in dictionary)
			{
				dictionary2.Add(item.Key, item.Value);
			}
			dictionary2.Add(key, value);
			return dictionary2;
		}

		public static void UpdateNameDictionary(string key, string value, bool sorting, bool isNameChinh, bool writeToFile = true)
		{
			string value2 = value.Split('/', '|')[0];
			vietPhraseDictionary[key] = value;
			vietPhraseOneMeaningDictionary[key] = value2;
			Dictionary<string, string> dictionary = (isNameChinh ? onlyNameChinhDictionary : onlyNamePhuDictionary);
			bool flag = dictionary.ContainsKey(key);
			if (flag)
			{
				dictionary[key] = value;
			}
			else if (sorting)
			{
				dictionary.Add(key, value);
			}
			else if (isNameChinh)
			{
				onlyNameChinhDictionary = AddEntryToDictionaryWithoutSorting(onlyNameChinhDictionary, key, value);
				dictionary = onlyNameChinhDictionary;
			}
			else
			{
				onlyNamePhuDictionary = AddEntryToDictionaryWithoutSorting(onlyNamePhuDictionary, key, value);
				dictionary = onlyNamePhuDictionary;
			}
			writeNamesHistoryLog(key, flag ? "Updated" : "Added", isNameChinh);
			if (onlyNameDictionary.ContainsKey(key))
			{
				onlyNameDictionary[key] = value;
				onlyNameOneMeaningDictionary[key] = value2;
			}
			else if (sorting)
			{
				onlyNameDictionary.Add(key, value);
				onlyNameOneMeaningDictionary.Add(key, value2);
			}
			else
			{
				onlyNameDictionary = AddEntryToDictionaryWithoutSorting(onlyNameDictionary, key, value);
				onlyNameOneMeaningDictionary = AddEntryToDictionaryWithoutSorting(onlyNameOneMeaningDictionary, key, value2);
			}
			if (writeToFile)
			{
				string filePath = (isNameChinh ? DictionaryConfigurationHelper.GetNamesDictionaryPath() : DictionaryConfigurationHelper.GetNamesPhuDictionaryPath());
				if (sorting)
				{
					SaveDictionaryToFile(ref dictionary, filePath);
				}
				else
				{
					SaveDictionaryToFileWithoutSorting(dictionary, filePath);
				}
			}
		}

		public static void UpdatePhienAmDictionary(string key, string value)
		{
			if (hanVietDictionary.ContainsKey(key))
			{
				hanVietDictionary[key] = value;
				writePhienAmHistoryLog(key, "Updated");
			}
			else
			{
				hanVietDictionary.Add(key, value);
				writePhienAmHistoryLog(key, "Added");
			}
			SaveDictionaryToFile(ref hanVietDictionary, DictionaryConfigurationHelper.GetChinesePhienAmWordsDictionaryPath());
			MarkCacheAsDirty(DCType.HanViet);
		}

		public static void DeleteKeyFromPhienAmDictionary(string key)
		{
			hanVietDictionary.Remove(key);
			SaveDictionaryToFile(ref hanVietDictionary, DictionaryConfigurationHelper.GetChinesePhienAmWordsDictionaryPath());
			MarkCacheAsDirty(DCType.HanViet);
			writePhienAmHistoryLog(key, "Deleted");
		}

		public static void UpdateLuatNhanDictionary(string key, string value)
		{
			luatNhanDictionary[key] = value;
			SaveDictionaryToFile(ref luatNhanDictionary, DictionaryConfigurationHelper.GetLuatNhanDictionaryPath());
		}

		public static void DeleteKeyFromLuatNhanDictionary(string key)
		{
			luatNhanDictionary.Remove(key);
			SaveDictionaryToFile(ref luatNhanDictionary, DictionaryConfigurationHelper.GetLuatNhanDictionaryPath());
		}

		public static void UpdateHauTuDictionary(string key, string value)
		{
			hauTuDictionary[key] = value;
			SaveDictionaryToFile(ref hauTuDictionary, DictionaryConfigurationHelper.GetHauTuDictionaryPath());
			MarkCacheAsDirty(DCType.HauTu);
		}

		public static void DeleteKeyFromHauTuDictionary(string key)
		{
			hauTuDictionary.Remove(key);
			SaveDictionaryToFile(ref hauTuDictionary, DictionaryConfigurationHelper.GetHauTuDictionaryPath());
			MarkCacheAsDirty(DCType.HauTu);
		}

		public static void UpdateDanhTuDictionary(string key, string value)
		{
			danhTuDictionary[key] = value;
			SaveDictionaryToFile(ref danhTuDictionary, DictionaryConfigurationHelper.GetDanhTuDictionaryPath());
			MarkCacheAsDirty(DCType.DanhTu);
		}

		public static void DeleteKeyFromDanhTuDictionary(string key)
		{
			danhTuDictionary.Remove(key);
			SaveDictionaryToFile(ref danhTuDictionary, DictionaryConfigurationHelper.GetDanhTuDictionaryPath());
			MarkCacheAsDirty(DCType.DanhTu);
		}

		public static void UpdateHoNguoiDictionary(string key, string value)
		{
			hoNguoiDictionary[key] = value;
			SaveDictionaryToFile(ref hoNguoiDictionary, DictionaryConfigurationHelper.GetHoNguoiDictionaryPath());
			MarkCacheAsDirty(DCType.HoNguoi);
		}

		public static void DeleteKeyFromHoNguoiDictionary(string key)
		{
			hoNguoiDictionary.Remove(key);
			SaveDictionaryToFile(ref hoNguoiDictionary, DictionaryConfigurationHelper.GetHoNguoiDictionaryPath());
			MarkCacheAsDirty(DCType.HoNguoi);
		}

		public static void SaveDictionaryToFileWithoutSorting(Dictionary<string, string> dictionary, string filePath)
		{
			string text = filePath + "." + DateTime.Now.Ticks;
			if (File.Exists(filePath))
			{
				File.Copy(filePath, text, overwrite: true);
			}
			StringBuilder stringBuilder = new StringBuilder();
			foreach (KeyValuePair<string, string> item in dictionary)
			{
				stringBuilder.Append(item.Key).Append("=").AppendLine(item.Value);
			}
			try
			{
				File.WriteAllText(filePath, stringBuilder.ToString(), Encoding.UTF8);
			}
			catch (Exception ex)
			{
				try
				{
					File.Copy(text, filePath, overwrite: true);
				}
				catch
				{
				}
				throw ex;
			}
			if (File.Exists(filePath))
			{
				File.Delete(text);
			}
		}

		public static void SaveDictionaryToFile(ref Dictionary<string, string> dictionary, string filePath)
		{
			IOrderedEnumerable<KeyValuePair<string, string>> orderedEnumerable = from pair in dictionary
				orderby GetNormalizedLength(pair.Key) descending, pair.Key
				select pair;
			Dictionary<string, string> dictionary2 = new Dictionary<string, string>();
			string text = filePath + "." + DateTime.Now.Ticks;
			if (File.Exists(filePath))
			{
				File.Copy(filePath, text, overwrite: true);
			}
			StringBuilder stringBuilder = new StringBuilder();
			foreach (KeyValuePair<string, string> item in orderedEnumerable)
			{
				stringBuilder.Append(item.Key).Append("=").AppendLine(item.Value);
				dictionary2.Add(item.Key, item.Value);
			}
			dictionary = dictionary2;
			try
			{
				File.WriteAllText(filePath, stringBuilder.ToString(), Encoding.UTF8);
				if (File.Exists(text))
				{
					File.Delete(text);
				}
			}
			catch
			{
				try
				{
					if (File.Exists(text))
					{
						File.Copy(text, filePath, overwrite: true);
					}
				}
				catch
				{
				}
			}
		}

		private static int GetNormalizedLength(string key)
		{
			string pattern = "\\([^)]*\\)|\\[[^\\]]*\\]";
			MatchEvaluator evaluator = delegate(Match match)
			{
				string value = match.Value;
				if (value.StartsWith("("))
				{
					string text = value.Substring(1, value.Length - 2);
					if (text.Contains("|"))
					{
						int count = text.Split(new char[1] { '|' }).Min((string option) => option.Length);
						return new string('c', count);
					}
					return new string('c', text.Length);
				}
				return value.StartsWith("[") ? "c" : value;
			};
			return Regex.Replace(key, pattern, evaluator).Replace("?", "").Length;
		}

		public static string ChineseToHanViet(string chinese, out CharRange[] chineseHanVietMappingArray)
		{
			LastTranslatedWord_HanViet = "";
			List<CharRange> list = new List<CharRange>();
			resultHanViet.Clear();
			int length = chinese.Length;
			for (int i = 0; i < length - 1; i++)
			{
				int startIndexOfNextTranslatedText = resultHanViet.Length;
				char c = chinese[i];
				char character = chinese[i + 1];
				if (isChinese(c))
				{
					if (isChinese(character))
					{
						appendTranslatedWord(resultHanViet, ChineseToHanViet(c), ref LastTranslatedWord_HanViet, ref startIndexOfNextTranslatedText);
						resultHanViet.Append(" ");
						LastTranslatedWord_HanViet += " ";
						list.Add(new CharRange(startIndexOfNextTranslatedText, ChineseToHanViet(c).Length));
					}
					else
					{
						appendTranslatedWord(resultHanViet, ChineseToHanViet(c), ref LastTranslatedWord_HanViet, ref startIndexOfNextTranslatedText);
						list.Add(new CharRange(startIndexOfNextTranslatedText, ChineseToHanViet(c).Length));
					}
				}
				else
				{
					resultHanViet.Append(c);
					LastTranslatedWord_HanViet += c;
					list.Add(new CharRange(startIndexOfNextTranslatedText, 1));
				}
			}
			if (isChinese(chinese[length - 1]))
			{
				appendTranslatedWord(resultHanViet, ChineseToHanViet(chinese[length - 1]), ref LastTranslatedWord_HanViet);
				list.Add(new CharRange(resultHanViet.Length, ChineseToHanViet(chinese[length - 1]).Length));
			}
			else
			{
				resultHanViet.Append(chinese[length - 1]);
				LastTranslatedWord_HanViet += chinese[length - 1];
				list.Add(new CharRange(resultHanViet.Length, 1));
			}
			chineseHanVietMappingArray = list.ToArray();
			LastTranslatedWord_HanViet = "";
			return resultHanViet.ToString();
		}

		public static TranslationResult ChineseToVietPhrase(string chinese, int wrapType, int translationAlgorithm, bool prioritizedName, int scanRange)
		{
			return TranslateAll(chinese, wrapType, translationAlgorithm, prioritizedName, scanRange, vietPhraseDictionary, ref LastTranslatedWord_VietPhrase);
		}

		public static TranslationResult ChineseToVietPhraseOneMeaning(string chinese, int wrapType, int translationAlgorithm, bool prioritizedName, int scanRange)
		{
			return TranslateAll(chinese, wrapType, translationAlgorithm, prioritizedName, scanRange, vietPhraseOneMeaningDictionary, ref LastTranslatedWord_VietPhraseOneMeaning);
		}

		private static TranslationResult TranslateAll(string chinese, int wrapType, int translationAlgorithm, bool prioritizedName, int scanRange, Dictionary<string, string> dictionary, ref string lastTranslatedWord)
		{
			List<CharRange> list = new List<CharRange>();
			List<CharRange> list2 = new List<CharRange>();
			StringBuilder stringBuilder = new StringBuilder();
			lastTranslatedWord = "";
			int num = chinese.Length - 1;
			int num2 = 0;
			int num3 = -1;
			int num4 = -1;
			int num5 = -1;
			Dictionary<int, NumberInfo> dictionary2 = PreScanForNumbers(chinese).ToDictionary((NumberInfo n) => n.StartIndex);
			chinese = NumberModifier(chinese);
			while (num2 <= num)
			{
				bool flag = false;
				bool flag2 = true;
				dictionary2.TryGetValue(num2, out var value);
				for (int num6 = scanRange; num6 > 0; num6--)
				{
					if (num2 + num6 <= chinese.Length && (value == null || num6 >= value.Length))
					{
						string text = chinese.Substring(num2, num6);
						if (dictionary.TryGetValue(text, out var value2))
						{
							bool flag3 = isLongestPhraseInSentence(chinese, num2, num6, dictionary, translationAlgorithm);
							if ((!prioritizedName || !containsName(chinese, num2, num6)) && ((translationAlgorithm != 0 && translationAlgorithm != 2) || flag3 || (prioritizedName && onlyNameDictionary.ContainsKey(text))))
							{
								ProcessTranslation(chinese, text, value2, num2, num6, wrapType, ref flag, ref num2, list, list2, stringBuilder, ref lastTranslatedWord);
								break;
							}
						}
						else if (!text.Contains("\n") && !text.Contains("\t") && flag2 && 2 < num6 && num3 < num2 + num6 - 1)
						{
							HandleNhanBy(chinese, ref num2, num6, ref num3, ref num4, ref num5, ref flag2, wrapType, translationAlgorithm, list, list2, ref flag, dictionary, stringBuilder, ref lastTranslatedWord);
							if (flag)
							{
								break;
							}
						}
					}
				}
				if (flag)
				{
					continue;
				}
				if (!flag && value != null)
				{
					string empty = string.Empty;
					string text2 = ChineseToLuatNhanOneMeaning(value.Text, "{s}", empty);
					if (!string.IsNullOrEmpty(text2))
					{
						ProcessTranslation(chinese, value.Text, text2, num2, value.Length, wrapType, ref flag, ref num2, list, list2, stringBuilder, ref lastTranslatedWord);
					}
				}
				if (!flag && !flag)
				{
					ProcessHanViet(chinese, wrapType, ref num2, list, list2, stringBuilder, ref lastTranslatedWord);
				}
			}
			return new TranslationResult
			{
				TranslatedText = stringBuilder.ToString(),
				ChinesePhraseRanges = list.ToArray(),
				VietPhraseRanges = list2.ToArray()
			};
		}

		private static string FindHoHauTuPhrase(string chinese, int startIndex, out int bestHTLength)
		{
			bestHTLength = 0;
			for (int num = 6; num >= 2; num--)
			{
				if (startIndex + num <= chinese.Length)
				{
					string text = chinese.Substring(startIndex, num);
					if (hoHauTuCache.Contains(text) && !vietPhraseDictionary.ContainsKey(text))
					{
						bool flag = false;
						for (int i = num + 1; i <= 20 && startIndex + i <= chinese.Length; i++)
						{
							string key = chinese.Substring(startIndex, i);
							if (vietPhraseDictionary.ContainsKey(key))
							{
								flag = true;
								break;
							}
						}
						if (!flag)
						{
							bestHTLength = num;
							return text;
						}
					}
				}
			}
			return null;
		}

		private static List<NumberInfo> PreScanForNumbers(string text)
		{
			List<NumberInfo> list = new List<NumberInfo>();
			for (int i = 0; i < text.Length; i++)
			{
				if (NumberChars.Contains(text[i]))
				{
					int matchedLength;
					NumberType numberType;
					string text2 = TransLuatNhan.FindLongestNumber(text, i, out matchedLength, out numberType);
					if (text2 != null && !onlyVietPhraseDictionary.ContainsKey(text2))
					{
						list.Add(new NumberInfo
						{
							StartIndex = i,
							Length = matchedLength,
							Text = text2,
							Type = numberType
						});
						i += matchedLength - 1;
					}
				}
			}
			return list;
		}

		private static void HandleNhanBy(string chinese, ref int num2, int num6, ref int num3, ref int num4, ref int num5, ref bool flag2, int wrapType, int translationAlgorithm, List<CharRange> list, List<CharRange> list2, ref bool flag, Dictionary<string, string> dictionary, StringBuilder result, ref string lastTranslatedWord)
		{
			if (num2 < num4)
			{
				if (num4 < num2 + num6 && num6 <= num5 - num4)
				{
					num6 = num4 - num2 + 1;
				}
				return;
			}
			string luatNhan = string.Empty;
			string valueN = string.Empty;
			int matchedLength = -1;
			int num7 = containsLuatNhan(chinese.Substring(num2, num6), out luatNhan, out matchedLength, out valueN);
			num4 = num2 + num7;
			num5 = num4 + matchedLength;
			if (num7 == 0)
			{
				if (!luatNhan.Contains("{n}"))
				{
					bool flag3 = false;
					for (int i = 1; i < matchedLength; i++)
					{
						int num8 = num2 + i;
						for (int num9 = Math.Min(20, chinese.Length - num8); num9 > 1; num9--)
						{
							string key = chinese.Substring(num8, num9);
							if (onlyNameDictionary.ContainsKey(key))
							{
								flag3 = true;
								break;
							}
						}
						if (flag3)
						{
							break;
						}
					}
					if (flag3)
					{
						return;
					}
				}
				if (num2 + matchedLength <= chinese.Length)
				{
					string text = chinese.Substring(num2, matchedLength);
					string text2 = ChineseToLuatNhanOneMeaning(text, luatNhan, valueN);
					if (!string.IsNullOrEmpty(text2))
					{
						string translation = text2.Trim();
						ProcessTranslation(chinese, text, translation, num2, matchedLength, wrapType, ref flag, ref num2, list, list2, result, ref lastTranslatedWord);
					}
				}
			}
			else if (0 >= num7)
			{
				num3 = num2 + num6 - 1;
				flag2 = false;
				int j;
				for (j = 100; num2 + j < chinese.Length && isChinese(chinese[num2 + j - 1]); j++)
				{
				}
				if (num2 + j <= chinese.Length && containsLuatNhan(chinese.Substring(num2, j), out var _, out var _, out var _) < 0)
				{
					num3 = num2 + j - 1;
				}
			}
		}

		private static void ProcessTranslation(string chinese, string subString, string translation, int startIndex, int length, int wrapType, ref bool flag, ref int num2, List<CharRange> list, List<CharRange> list2, StringBuilder result, ref string lastTranslatedWord)
		{
			list.Add(new CharRange(startIndex, length));
			string text = WrapTranslation(translation, wrapType);
			appendTranslatedWord(result, text, ref lastTranslatedWord);
			list2.Add(new CharRange(result.Length - text.Length, text.Length));
			if (nextCharIsChinese(chinese, startIndex + length - 1))
			{
				result.Append(" ");
				lastTranslatedWord += " ";
			}
			flag = true;
			num2 += length;
		}

		private static void ProcessHanViet(string chinese, int wrapType, ref int num2, List<CharRange> list, List<CharRange> list2, StringBuilder result, ref string lastTranslatedWord)
		{
			int length = result.Length;
			int num3 = ChineseToHanViet(chinese[num2]).Length;
			list.Add(new CharRange(num2, 1));
			if (isChinese(chinese[num2]))
			{
				string translatedText = WrapTranslation(ChineseToHanViet(chinese[num2]), wrapType);
				appendTranslatedWord(result, translatedText, ref lastTranslatedWord);
				if (nextCharIsChinese(chinese, num2))
				{
					result.Append(" ");
					lastTranslatedWord += " ";
				}
				num3 += ((wrapType == 1) ? 2 : 0);
			}
			else if ((chinese[num2] == '"' || chinese[num2] == '\'') && !lastTranslatedWord.EndsWith(" ") && !lastTranslatedWord.EndsWith(".") && !lastTranslatedWord.EndsWith("?") && !lastTranslatedWord.EndsWith("!") && !lastTranslatedWord.EndsWith("\t") && num2 < chinese.Length - 1 && chinese[num2 + 1] != ' ' && chinese[num2 + 1] != ',')
			{
				result.Append(" ").Append(chinese[num2]);
				lastTranslatedWord = lastTranslatedWord + " " + chinese[num2];
			}
			else
			{
				result.Append(chinese[num2]);
				lastTranslatedWord += chinese[num2];
				num3 = 1;
			}
			list2.Add(new CharRange(length, num3));
			num2++;
		}

		private static string WrapTranslation(string translation, int wrapType)
		{
			if (wrapType == 0)
			{
				return translation;
			}
			return "[" + translation + "]";
		}

		public static string ChineseToHanViet(char chinese)
		{
			if (chinese == ' ')
			{
				return "";
			}
			if (hanVietDictionary.TryGetValue(chinese.ToString(), out var value))
			{
				return value;
			}
			return ToNarrow(chinese.ToString());
		}

		public static string ChineseToMeanings(string chinese, out int phraseLength)
		{
			phraseLength = 0;
			if (string.IsNullOrEmpty(chinese))
			{
				return "";
			}
			string text = "";
			string text2 = ((chinese.Length > 20) ? chinese.Substring(0, 20) : chinese);
			string text3 = NumberModifier(text2);
			bool flag = false;
			string valueN = string.Empty;
			if (containsLuatNhan(text3, out var luatNhan, out var matchedLength, out valueN) == 0 && matchedLength > 0)
			{
				string text4 = text3.Substring(0, matchedLength);
				string text5 = ChineseToLuatNhanOneMeaning(text4, luatNhan, valueN);
				if (!string.IsNullOrEmpty(text5))
				{
					text = text4 + " (Luật Nhân) " + text5 + "\n-----------------\n";
					flag = true;
					if (phraseLength == 0)
					{
						phraseLength = matchedLength;
					}
				}
			}
			if (!flag)
			{
				NumberInfo numberInfo = (from n in PreScanForNumbers(text3)
					where n.StartIndex == 0
					orderby n.Length descending
					select n).FirstOrDefault();
				if (numberInfo != null)
				{
					string text6 = numberInfo.Text;
					string text7 = ChineseToLuatNhanOneMeaning(text6, "{s}", valueN);
					if (!string.IsNullOrEmpty(text7))
					{
						text = text + text6 + " (Luật S) " + text7 + "\n-----------------\n";
						phraseLength = numberInfo.Length;
					}
				}
			}
			for (int num = Math.Min(20, text2.Length); num > 0; num--)
			{
				string text8 = text2.Substring(0, num);
				if (onlyNamePhuDictionary.TryGetValue(text8, out var value))
				{
					text = text + text8 + " (Names phụ) " + value.Replace("/", "; ") + "\n-----------------\n";
					if (phraseLength == 0)
					{
						phraseLength = num;
					}
				}
				if (onlyNameChinhDictionary.TryGetValue(text8, out var value2))
				{
					text = text + text8 + " (Name chính) " + value2.Replace("/", "; ") + "\n-----------------\n";
					if (phraseLength == 0)
					{
						phraseLength = num;
					}
				}
				if (onlyVietPhraseDictionary.TryGetValue(text8, out var value3))
				{
					text = text + text8 + " (VietPhrase) " + value3.Replace("/", "; ") + "\n-----------------\n";
					if (phraseLength == 0)
					{
						phraseLength = num;
					}
				}
			}
			for (int num2 = Math.Min(20, text2.Length); num2 > 0; num2--)
			{
				string text9 = text2.Substring(0, num2);
				if (lacVietDictionary.TryGetValue(text9, out var value4))
				{
					text = text + text9 + " (Lạc Việt)\n" + value4 + "\n-----------------\n";
					if (phraseLength == 0)
					{
						phraseLength = 1;
					}
				}
			}
			if (phraseLength == 0)
			{
				phraseLength = 1;
				text = $"{text2[0]}\n-----------------\nNot Found";
			}
			return text;
		}

		private static string NumberModifier(string text)
		{
			if (string.IsNullOrEmpty(text))
			{
				return text;
			}
			StringBuilder stringBuilder = new StringBuilder(text.Length);
			int length = text.Length;
			int num = 0;
			while (num < length)
			{
				char c = text[num];
				if ((c == '余' || c == '多') && num + 1 < length)
				{
					char value = text[num + 1];
					if (Enumerable.Contains("百千万亿", value))
					{
						stringBuilder.Append(value);
						stringBuilder.Append(c);
						num += 2;
						continue;
					}
				}
				stringBuilder.Append(c);
				num++;
			}
			return stringBuilder.ToString();
		}

		public async Task ForceReloadDictionaries()
		{
			dictionaryDirty = true;
			await LoadDictionaries();
		}

		public async Task LoadDictionaries()
		{
			lock (lockObject)
			{
				if (!dictionaryDirty || isLoading)
				{
					return;
				}
				isLoading = true;
			}
			try
			{
				await Task.WhenAll(new List<Task>
				{
					Task.Run(delegate
					{
						loadHanVietDictionary();
					}),
					Task.Run(delegate
					{
						loadLacVietDictionary();
					}),
					Task.Run(delegate
					{
						loadIgnoredChinesePhraseLists();
					}),
					Task.Run(delegate
					{
						loadOnlyNameDictionaryHistory();
					}),
					Task.Run(delegate
					{
						loadOnlyNamePhuDictionaryHistory();
					}),
					Task.Run(delegate
					{
						loadOnlyVietPhraseDictionaryHistory();
					}),
					Task.Run(delegate
					{
						loadHanVietDictionaryHistory();
					}),
					Task.Run(delegate
					{
						loadOnlyVietPhraseDictionary();
					}),
					Task.Run(delegate
					{
						loadOnlyNameDictionary();
					}),
					Task.Run(delegate
					{
						loadLuatNhanDictionary();
					}),
					Task.Run(delegate
					{
						loadPronounDictionary();
					}),
					Task.Run(delegate
					{
						loadDanhTuDictionary();
					}),
					Task.Run(delegate
					{
						loadHoNguoiDictionary();
					}),
					Task.Run(delegate
					{
						loadHauTuDictionary();
					})
				});
				loadVietPhraseDictionary();
				vPDictToVPOneMeaningDict();
				UpdateDictionaryN();
				UpdateLuatNhanListsAndCaches();
				BuildHoHauTuCache();
				lock (lockObject)
				{
					dictionaryDirty = false;
				}
				this.DataDictionaryLoaded?.Invoke(this, new DictionaryEventArgs(onlyVietPhraseDictionary.Count, onlyNameChinhDictionary.Count, onlyNamePhuDictionary.Count));
			}
			catch
			{
				lock (lockObject)
				{
					dictionaryDirty = true;
				}
			}
			finally
			{
				lock (lockObject)
				{
					isLoading = false;
				}
			}
		}

		private static void UpdateDictionaryN()
		{
			lock (lockObject)
			{
				dictionaryN = new Dictionary<string, string>(pronounDictionary);
				AddToDictionary(dictionaryN, onlyNameOneMeaningDictionary);
			}
		}

		private static void UpdateLuatNhanListsAndCaches()
		{
			lock (lockObject)
			{
				luatNhanNList = luatNhanDictionary.Where((KeyValuePair<string, string> kv) => kv.Key.Contains("{n}") && !kv.Key.Contains("{s}")).ToDictionary((KeyValuePair<string, string> kv) => kv.Key, (KeyValuePair<string, string> kv) => kv.Value);
				luatNhanSList = luatNhanDictionary.Where((KeyValuePair<string, string> kv) => kv.Key.Contains("{s}") && !kv.Key.Contains("{n}")).ToDictionary((KeyValuePair<string, string> kv) => kv.Key, (KeyValuePair<string, string> kv) => kv.Value);
				luatNhanSCache.Clear();
				foreach (string key in luatNhanSList.Keys)
				{
					if (!(key == "{s}"))
					{
						string text = Regex.Replace(key.Replace("(", "(?:").Replace("{s}", " {s} ").Trim(), "\\s+", "\\s*");
						string text2 = text.Replace("{s}", "((?:\\d+\\s*[万亿])|(?:\\d+)|(?:[零一二三四五六七八九十百千万亿两〇]+))");
						if (key == "{s}两")
						{
							text2 += "(?!(?:[零一二三四五六七八九十百千万亿两〇\\d]+){1,2})";
						}
						if (key == "百分[之]?{s}")
						{
							text2 = text.Replace("{s}", "([零一二三四五六七八九十百千万亿两〇点\\d]+)");
						}
						luatNhanSCache[key] = new Regex(text2, RegexOptions.Compiled);
					}
				}
				luatNhanNCache.Clear();
				foreach (string key2 in luatNhanNList.Keys)
				{
					string text3 = key2.Replace("(", "(?:");
					int num = text3.IndexOf("{n}");
					string pattern;
					if (num >= 0 && num + 3 < text3.Length)
					{
						string text4 = text3.Substring(0, num);
						string text5 = text3.Substring(num + 3);
						string text6 = Regex.Escape(text5);
						string text7 = "((?:(?!" + text6 + ")[^,\\. ?]){1,10}?)";
						pattern = text4 + text7 + text5;
					}
					else
					{
						pattern = text3.Replace("{n}", "([^,\\. ?]{1,10})");
					}
					luatNhanNCache[key2] = new Regex(pattern, RegexOptions.Compiled);
				}
			}
		}

		private static void BuildHoHauTuCache()
		{
			if (changeCounters[DCType.HoNguoi] == 0 && changeCounters[DCType.HauTu] == 0)
			{
				return;
			}
			lock (lockObject)
			{
				hoHauTuCache.Clear();
				if (hoNguoiDictionary == null || hauTuDictionary == null)
				{
					return;
				}
				foreach (string key in hoNguoiDictionary.Keys)
				{
					foreach (string key2 in hauTuDictionary.Keys)
					{
						hoHauTuCache.Add(key + key2);
					}
				}
			}
			changeCounters[DCType.HoNguoi] = 0;
			changeCounters[DCType.HauTu] = 0;
		}

		private static void loadLuatNhanDictionary()
		{
			Dictionary<string, string> dictionary = new Dictionary<string, string>();
			using (TextReader textReader = new StreamReader(DictionaryConfigurationHelper.GetLuatNhanDictionaryPath(), detectEncodingFromByteOrderMarks: true))
			{
				string text;
				while ((text = textReader.ReadLine()) != null)
				{
					if (!text.StartsWith("#"))
					{
						string[] array = text.Split(new char[1] { '=' });
						if (array.Length == 2 && !dictionary.ContainsKey(array[0]))
						{
							dictionary.Add(array[0], array[1]);
						}
					}
				}
			}
			IOrderedEnumerable<KeyValuePair<string, string>> orderedEnumerable = dictionary.OrderByDescending(delegate(KeyValuePair<string, string> pair)
			{
				KeyValuePair<string, string> keyValuePair2 = pair;
				return keyValuePair2.Key.Length;
			}).ThenBy(delegate(KeyValuePair<string, string> pair)
			{
				KeyValuePair<string, string> keyValuePair = pair;
				return keyValuePair.Key;
			});
			luatNhanDictionary.Clear();
			foreach (KeyValuePair<string, string> item in orderedEnumerable)
			{
				luatNhanDictionary.Add(item.Key, item.Value);
			}
		}

		private static void loadVietPhraseDictionary()
		{
			vietPhraseDictionary.Clear();
			foreach (KeyValuePair<string, string> item in onlyNameDictionary)
			{
				if (!vietPhraseDictionary.ContainsKey(item.Key))
				{
					vietPhraseDictionary.Add(item.Key, item.Value);
				}
			}
			foreach (KeyValuePair<string, string> item2 in onlyVietPhraseDictionary)
			{
				if (!vietPhraseDictionary.ContainsKey(item2.Key))
				{
					vietPhraseDictionary.Add(item2.Key, item2.Value);
				}
			}
		}

		private static void loadOnlyNameDictionary()
		{
			onlyNameDictionary.Clear();
			onlyNameOneMeaningDictionary.Clear();
			onlyNameChinhDictionary.Clear();
			onlyNamePhuDictionary.Clear();
			char[] separator = "/|".ToCharArray();
			using (TextReader textReader = new StreamReader(DictionaryConfigurationHelper.GetNamesDictionaryPath(), detectEncodingFromByteOrderMarks: true))
			{
				string text;
				while ((text = textReader.ReadLine()) != null)
				{
					string[] array = text.Split(new char[1] { '=' });
					if (array.Length == 2 && !onlyNameDictionary.ContainsKey(array[0]))
					{
						onlyNameDictionary.Add(array[0], array[1]);
						onlyNameOneMeaningDictionary.Add(array[0], array[1].Split(separator)[0]);
						onlyNameChinhDictionary.Add(array[0], array[1]);
					}
				}
			}
			using TextReader textReader2 = new StreamReader(DictionaryConfigurationHelper.GetNamesPhuDictionaryPath(), detectEncodingFromByteOrderMarks: true);
			string text2;
			while ((text2 = textReader2.ReadLine()) != null)
			{
				string[] array2 = text2.Split(new char[1] { '=' });
				if (array2.Length == 2 && !onlyNamePhuDictionary.ContainsKey(array2[0]))
				{
					string key = array2[0];
					string text3 = array2[1];
					onlyNameDictionary[key] = text3;
					onlyNameOneMeaningDictionary[key] = text3.Split(separator)[0];
					onlyNamePhuDictionary.Add(key, text3);
				}
			}
		}

		private static void vPDictToVPOneMeaningDict()
		{
			vietPhraseOneMeaningDictionary.Clear();
			foreach (KeyValuePair<string, string> item in vietPhraseDictionary)
			{
				Dictionary<string, string> dictionary = vietPhraseOneMeaningDictionary;
				string key = item.Key;
				string value = ((item.Value.Contains("/") || item.Value.Contains("|")) ? item.Value.Split('/', '|')[0] : item.Value);
				dictionary.Add(key, value);
			}
		}

		private static void AddToDictionary(Dictionary<string, string> target, Dictionary<string, string> source)
		{
			foreach (KeyValuePair<string, string> item in source)
			{
				if (!target.ContainsKey(item.Key))
				{
					target.Add(item.Key, item.Value);
				}
			}
		}

		private static void loadOnlyVietPhraseDictionaryHistory()
		{
			LoadDictionaryHistory(DictionaryConfigurationHelper.GetVietPhraseDictionaryHistoryPath(), ref onlyVietPhraseDictionaryHistoryDataSet);
		}

		private static void loadOnlyNameDictionaryHistory()
		{
			LoadDictionaryHistory(DictionaryConfigurationHelper.GetNamesDictionaryHistoryPath(), ref onlyNameDictionaryHistoryDataSet);
		}

		private static void loadOnlyNamePhuDictionaryHistory()
		{
			LoadDictionaryHistory(DictionaryConfigurationHelper.GetNamesPhuDictionaryHistoryPath(), ref onlyNamePhuDictionaryHistoryDataSet);
		}

		private static void loadHanVietDictionaryHistory()
		{
			if (changeCounters[DCType.HanViet] != 0)
			{
				LoadDictionaryHistory(DictionaryConfigurationHelper.GetChinesePhienAmWordsDictionaryHistoryPath(), ref hanVietDictionaryHistoryDataSet);
				changeCounters[DCType.HanViet] = 0;
			}
		}

		public static void LoadDictionaryHistory(string dictionaryHistoryPath, ref DataSet dictionaryHistoryDataSet)
		{
			EnsureDictionaryHistoryTableExists(dictionaryHistoryDataSet);
			DataTable dataTable = dictionaryHistoryDataSet.Tables["DictionaryHistory"];
			dataTable.Clear();
			if (!File.Exists(dictionaryHistoryPath))
			{
				return;
			}
			foreach (string item in File.ReadLines(dictionaryHistoryPath, Encoding.GetEncoding(CharsetDetector.DetectChineseCharset(dictionaryHistoryPath))).Skip(1))
			{
				string[] array = item.Split(new char[1] { '\t' });
				if (array.Length == 4 && !string.IsNullOrWhiteSpace(array[0]) && DateTime.TryParseExact(array[3], "yyyy-MM-dd HH:mm:ss", CultureInfo.InvariantCulture, DateTimeStyles.None, out var result))
				{
					DataRow dataRow = dataTable.Rows.Find(array[0]);
					if (dataRow == null)
					{
						dataTable.Rows.Add(array[0], array[1], array[2], result);
					}
					else
					{
						dataRow["Action"] = array[1];
						dataRow["User Name"] = array[2];
						dataRow["Updated Date"] = result;
					}
				}
			}
		}

		private static void EnsureDictionaryHistoryTableExists(DataSet dataSet)
		{
			if (!dataSet.Tables.Contains("DictionaryHistory"))
			{
				DataTable dataTable = new DataTable("DictionaryHistory");
				dataTable.Columns.Add("Entry", typeof(string));
				dataTable.Columns.Add("Action", typeof(string));
				dataTable.Columns.Add("User Name", typeof(string));
				dataTable.Columns.Add("Updated Date", typeof(DateTime));
				dataTable.PrimaryKey = new DataColumn[1] { dataTable.Columns["Entry"] };
				dataSet.Tables.Add(dataTable);
			}
		}

		private static void LoadDictionaryCL(string filePath, Dictionary<string, string> dictionary)
		{
			dictionary.Clear();
			using TextReader textReader = new StreamReader(filePath, detectEncodingFromByteOrderMarks: true);
			string text;
			while ((text = textReader.ReadLine()) != null)
			{
				string[] array = text.Split(new char[1] { '=' });
				if (array.Length == 2 && !dictionary.ContainsKey(array[0]))
				{
					dictionary.Add(array[0], array[1]);
				}
			}
		}

		private static void loadHanVietDictionary()
		{
			if (changeCounters[DCType.HanViet] != 0)
			{
				LoadDictionaryCL(DictionaryConfigurationHelper.GetChinesePhienAmWordsDictionaryPath(), hanVietDictionary);
			}
		}

		private static void loadDanhTuDictionary()
		{
			if (changeCounters[DCType.DanhTu] != 0)
			{
				LoadDictionaryCL(DictionaryConfigurationHelper.GetDanhTuDictionaryPath(), danhTuDictionary);
				changeCounters[DCType.DanhTu] = 0;
			}
		}

		private static void loadHoNguoiDictionary()
		{
			if (changeCounters[DCType.HoNguoi] != 0)
			{
				LoadDictionaryCL(DictionaryConfigurationHelper.GetHoNguoiDictionaryPath(), hoNguoiDictionary);
			}
		}

		private static void loadHauTuDictionary()
		{
			if (changeCounters[DCType.HauTu] != 0)
			{
				LoadDictionaryCL(DictionaryConfigurationHelper.GetHauTuDictionaryPath(), hauTuDictionary);
			}
		}

		private static void loadOnlyVietPhraseDictionary()
		{
			LoadDictionaryCL(DictionaryConfigurationHelper.GetVietPhraseDictionaryPath(), onlyVietPhraseDictionary);
		}

		private static void loadLacVietDictionary()
		{
			if (changeCounters[DCType.LacViet] != 0)
			{
				LoadDictionaryCL(DictionaryConfigurationHelper.GetLacVietDictionaryPath(), lacVietDictionary);
				changeCounters[DCType.LacViet] = 0;
			}
		}

		private static void loadPronounDictionary()
		{
			if (changeCounters[DCType.Pronoun] != 0)
			{
				LoadDictionaryCL(DictionaryConfigurationHelper.GetPronounsDictionaryPath(), pronounDictionary);
				changeCounters[DCType.Pronoun] = 0;
			}
		}

		public static void AddIgnoredChinesePhrase(string ignoredChinesePhrase)
		{
			if (!ignoredChinesePhraseList.Contains(ignoredChinesePhrase))
			{
				ignoredChinesePhraseList.Add(ignoredChinesePhrase);
				try
				{
					File.WriteAllLines(DictionaryConfigurationHelper.GetIgnoredChinesePhraseListPath(), ignoredChinesePhraseList.ToArray(), Encoding.UTF8);
				}
				catch
				{
				}
				loadIgnoredChinesePhraseLists();
			}
		}

		private static void loadIgnoredChinesePhraseLists()
		{
			ignoredChinesePhraseList.Clear();
			ignoredChinesePhraseForBrowserList.Clear();
			char[] trimChars = "\t\n".ToCharArray();
			using (TextReader textReader = new StreamReader(DictionaryConfigurationHelper.GetIgnoredChinesePhraseListPath(), detectEncodingFromByteOrderMarks: true))
			{
				string text;
				while ((text = textReader.ReadLine()) != null)
				{
					if (!string.IsNullOrEmpty(text))
					{
						string text2 = standardizeInputWithoutRemovingIgnoredChinesePhrases(text, addLineBreaks: true).Trim(trimChars);
						if (!string.IsNullOrEmpty(text2) && !ignoredChinesePhraseList.Contains(text2))
						{
							ignoredChinesePhraseList.Add(text2);
						}
						string text3 = standardizeInputForBrowserWithoutRemovingIgnoredChinesePhrases(text).Trim(trimChars);
						if (!string.IsNullOrEmpty(text3) && !ignoredChinesePhraseForBrowserList.Contains(text3))
						{
							ignoredChinesePhraseForBrowserList.Add(text3);
						}
					}
				}
			}
			ignoredChinesePhraseList.Sort(compareStringByDescending);
			ignoredChinesePhraseForBrowserList.Sort(compareStringByDescending);
		}

		private static int compareStringByDescending(string x, string y)
		{
			if (x == null)
			{
				if (y != null)
				{
					return 1;
				}
				return 0;
			}
			if (y == null)
			{
				return -1;
			}
			int num = x.Length.CompareTo(y.Length);
			if (num == 0)
			{
				return x.CompareTo(y) * -1;
			}
			return num * -1;
		}

		public static string StandardizeInput(string original, bool addLineBreaks)
		{
			return removeIgnoredChinesePhrases(standardizeInputWithoutRemovingIgnoredChinesePhrases(original, addLineBreaks));
		}

		private static string standardizeInputWithoutRemovingIgnoredChinesePhrases(string original, bool addLineBreaks)
		{
			if (string.IsNullOrEmpty(original))
			{
				return "";
			}
			string value = ToSimplified(original);
			value = WebUtility.HtmlDecode(value);
			string[] array = new string[16]
			{
				"，", "。", "：", "“", "”", "‘", "’", "？", "！", "「",
				"」", "．", "、", "\u3000", "…", NULL_STRING
			};
			string[] array2 = new string[16]
			{
				", ", ".", ": ", "\"", "\" ", "'", "' ", "?", "!", "\"",
				"\" ", ".", ", ", " ", "...", ""
			};
			for (int i = 0; i < array.Length; i++)
			{
				value = value.Replace(array[i], array2[i]);
			}
			string text = ToNarrow(value.Replace("  ", " ").Replace(" \r\n", "\n").Replace(" \n", "\n")
				.Replace(" ,", ","));
			int length = text.Length;
			StringBuilder stringBuilder = new StringBuilder();
			for (int j = 0; j < length - 1; j++)
			{
				char c = text[j];
				char c2 = text[j + 1];
				if (char.IsControl(c) && c != '\t' && c != '\n' && c != '\r')
				{
					continue;
				}
				if (char.IsDigit(c) && c2 == '》')
				{
					stringBuilder.Append(c).Append(" ");
					continue;
				}
				if (c == '.' && j > 0 && char.IsDigit(text[j - 1]) && char.IsDigit(c2))
				{
					stringBuilder.Append(c);
					continue;
				}
				if (isChinese(c))
				{
					if (!isChinese(c2) && c2 != ',' && c2 != '.' && c2 != ':' && c2 != ';' && c2 != '"' && c2 != '\'' && c2 != '?' && c2 != ' ' && c2 != '!' && c2 != ')' && c2 != '\n')
					{
						stringBuilder.Append(c).Append(" ");
					}
					else
					{
						stringBuilder.Append(c);
					}
					continue;
				}
				switch (c)
				{
				case '\t':
				case '\n':
				case ' ':
				case '"':
				case '\'':
				case '(':
					stringBuilder.Append(c);
					break;
				case '!':
				case '.':
				case '?':
					if (c2 == '"' || c2 == ' ' || c2 == '\'')
					{
						stringBuilder.Append(c);
					}
					else
					{
						stringBuilder.Append(c).Append(" ");
					}
					break;
				default:
					if (isChinese(c2))
					{
						stringBuilder.Append(c).Append(" ");
					}
					else
					{
						stringBuilder.Append(c);
					}
					break;
				}
			}
			stringBuilder.Append(text[length - 1]);
			string text2 = stringBuilder.ToString();
			if (!addLineBreaks)
			{
				return text2.Replace(". . . . . .", "...");
			}
			return indentAllLines(text2, insertBlankLine: true).Replace(". . . . . .", "...");
		}

		private static string standardizeInputForBrowserWithoutRemovingIgnoredChinesePhrases(string original)
		{
			if (string.IsNullOrEmpty(original))
			{
				return "";
			}
			string text = ToSimplified(original);
			string[] array = new string[16]
			{
				"，", "。", "：", "“", "”", "‘", "’", "？", "！", "「",
				"」", "．", "、", "\u3000", "…", NULL_STRING
			};
			string[] array2 = new string[16]
			{
				", ", ".", ": ", "\"", "\" ", "'", "' ", "?", "!", "\"",
				"\" ", ".", ", ", " ", "...", ""
			};
			for (int i = 0; i < array.Length; i++)
			{
				text = text.Replace(array[i], array2[i]);
			}
			string text2 = ToNarrow(text.Replace("  ", " ").Replace(" \r\n", "\n").Replace(" \n", "\n"));
			int length = text2.Length;
			StringBuilder stringBuilder = new StringBuilder();
			for (int j = 0; j < length - 1; j++)
			{
				char c = text2[j];
				char c2 = text2[j + 1];
				if (isChinese(c))
				{
					if (!isChinese(c2) && c2 != ',' && c2 != '.' && c2 != ':' && c2 != ';' && c2 != '"' && c2 != '\'' && c2 != '?' && c2 != ' ' && c2 != '!')
					{
						stringBuilder.Append(c).Append(" ");
					}
					else
					{
						stringBuilder.Append(c);
					}
				}
				else if (c == '\t' || c == ' ' || c == '"' || c == '\'' || c == '\n')
				{
					stringBuilder.Append(c);
				}
				else if (isChinese(c2))
				{
					stringBuilder.Append(c).Append(" ");
				}
				else
				{
					stringBuilder.Append(c);
				}
			}
			stringBuilder.Append(text2[length - 1]);
			return indentAllLines(stringBuilder.ToString());
		}

		public static string indentAllLines(string text, bool insertBlankLine)
		{
			string[] array = text.Split(new char[1] { '\n' }, StringSplitOptions.RemoveEmptyEntries);
			StringBuilder stringBuilder = new StringBuilder();
			string[] array2 = array;
			foreach (string text2 in array2)
			{
				if (!string.IsNullOrEmpty(text2.Trim()))
				{
					stringBuilder.Append("\t" + text2.Trim()).Append("\n").Append(insertBlankLine ? "\n" : "");
				}
			}
			return stringBuilder.ToString();
		}

		private static string indentAllLines(string text)
		{
			return indentAllLines(text, insertBlankLine: false);
		}

		private static bool isChinese(char character)
		{
			return hanVietDictionary.ContainsKey(character.ToString());
		}

		public static bool IsChinese(char character)
		{
			return isChinese(character);
		}

		public static bool IsAllChinese(string text)
		{
			for (int i = 0; i < text.Length; i++)
			{
				if (!isChinese(text[i]))
				{
					return false;
				}
			}
			return true;
		}

		public static string ToSimplified(string str)
		{
			return Strings.StrConv(str, (VbStrConv)256, 0);
		}

		internal static string ToWide(string str)
		{
			int length = str.Length;
			int i;
			for (i = 0; i < length; i++)
			{
				char c = str[i];
				if (c >= '!' && c <= '~')
				{
					break;
				}
			}
			if (i >= length)
			{
				return str;
			}
			StringBuilder stringBuilder = new StringBuilder();
			for (int j = 0; j < length; j++)
			{
				char c2 = str[j];
				if (c2 >= '!' && c2 <= '~')
				{
					stringBuilder.Append((char)(c2 - 33 + 65281));
				}
				else
				{
					stringBuilder.Append(c2);
				}
			}
			return stringBuilder.ToString();
		}

		internal static string ToNarrow(string str)
		{
			int length = str.Length;
			int i;
			for (i = 0; i < length; i++)
			{
				char c = str[i];
				if (c >= '！' && c <= '～')
				{
					break;
				}
			}
			if (i >= length)
			{
				return str;
			}
			StringBuilder stringBuilder = new StringBuilder();
			for (int j = 0; j < length; j++)
			{
				char c2 = str[j];
				if (c2 >= '！' && c2 <= '～')
				{
					stringBuilder.Append((char)(c2 - 65281 + 33));
				}
				else
				{
					stringBuilder.Append(c2);
				}
			}
			return stringBuilder.ToString();
		}

		public static void appendTranslatedWord(StringBuilder result, string translatedText, ref string lastTranslatedWord)
		{
			int startIndexOfNextTranslatedText = 0;
			appendTranslatedWord(result, translatedText, ref lastTranslatedWord, ref startIndexOfNextTranslatedText);
		}

		private static void appendTranslatedWord(StringBuilder result, string translatedText, ref string lastTranslatedWord, ref int startIndexOfNextTranslatedText)
		{
			lastTranslatedWord = ((lastTranslatedWord.EndsWith("\n") || lastTranslatedWord.EndsWith("\t") || lastTranslatedWord.EndsWith(". ") || lastTranslatedWord.EndsWith("\"") || lastTranslatedWord.EndsWith("'") || lastTranslatedWord.EndsWith("? ") || lastTranslatedWord.EndsWith("! ") || lastTranslatedWord.EndsWith(".\" ") || lastTranslatedWord.EndsWith("?\" ") || lastTranslatedWord.EndsWith("!\" ") || lastTranslatedWord.EndsWith(": ")) ? toUpperCase(translatedText) : ((lastTranslatedWord.EndsWith(" ") || lastTranslatedWord.EndsWith("(")) ? translatedText : (" " + translatedText)));
			if ((string.IsNullOrEmpty(translatedText) || translatedText[0] == ',' || translatedText[0] == '.' || translatedText[0] == '?' || translatedText[0] == '!') && 0 < result.Length && result[result.Length - 1] == ' ')
			{
				result = result.Remove(result.Length - 1, 1);
				startIndexOfNextTranslatedText--;
			}
			result.Append(lastTranslatedWord);
		}

		private static string toUpperCase(string text)
		{
			if (string.IsNullOrEmpty(text))
			{
				return text;
			}
			if (!text.StartsWith("[") || 2 > text.Length)
			{
				return string.Concat((object?)char.ToUpper(text[0]).ToString(), (object?)((text.Length <= 1) ? "" : text.Substring(1)));
			}
			return string.Concat((object?)"[", (object?)char.ToUpper(text[1]).ToString(), (object?)((text.Length <= 2) ? "" : text.Substring(2)));
		}

		private static bool nextCharIsChinese(string chinese, int currentPhraseEndIndex)
		{
			if (chinese.Length - 1 > currentPhraseEndIndex)
			{
				return isChinese(chinese[currentPhraseEndIndex + 1]);
			}
			return false;
		}

		public static bool IsInVietPhrase(string chinese)
		{
			return vietPhraseDictionary.ContainsKey(chinese);
		}

		public static string ChineseToHanVietForAnalyzer(string chinese)
		{
			if (string.IsNullOrEmpty(chinese))
			{
				return string.Empty;
			}
			StringBuilder stringBuilder = new StringBuilder(chinese.Length * 2);
			for (int i = 0; i < chinese.Length; i++)
			{
				char value = chinese[i];
				if (hanVietDictionary.TryGetValue(value.ToString(), out var value2))
				{
					stringBuilder.Append(value2).Append(' ');
				}
				else
				{
					stringBuilder.Append(value).Append(' ');
				}
			}
			return stringBuilder.ToString().TrimEnd(new char[0]);
		}

		public static string TranslateChineseToHanViet(string chinesePhrase)
		{
			List<string> list = new List<string>();
			for (int i = 0; i < chinesePhrase.Length; i++)
			{
				string text = chinesePhrase[i].ToString();
				if (hanVietDictionary.TryGetValue(text, out var value))
				{
					list.Add(value);
				}
				else
				{
					list.Add(text);
				}
			}
			return string.Join(" ", list);
		}

		private static bool containsName(string chinese, int startIndex, int phraseLength)
		{
			if (phraseLength < 2 || onlyNameDictionary.ContainsKey(chinese.Substring(startIndex, phraseLength)))
			{
				return false;
			}
			int num = startIndex + phraseLength - 1;
			int num2 = 2;
			for (int i = startIndex + 1; i <= num; i++)
			{
				for (int num3 = 20; num3 >= num2; num3--)
				{
					if (chinese.Length >= i + num3 && onlyNameDictionary.ContainsKey(chinese.Substring(i, num3)))
					{
						return true;
					}
				}
			}
			return false;
		}

		private static bool isLongestPhraseInSentence(string chinese, int startIndex, int phraseLength, Dictionary<string, string> dictionary, int translationAlgorithm)
		{
			if (phraseLength < 2)
			{
				return true;
			}
			int num = ((translationAlgorithm == 0) ? phraseLength : ((phraseLength < 3) ? 3 : phraseLength));
			int num2 = startIndex + phraseLength - 1;
			for (int i = startIndex + 1; i <= num2; i++)
			{
				for (int num3 = 20; num3 > num; num3--)
				{
					if (chinese.Length >= i + num3 && dictionary.ContainsKey(chinese.Substring(i, num3)))
					{
						return false;
					}
				}
			}
			return true;
		}

		public static int GetLuatNhanDictionaryCount()
		{
			return luatNhanDictionary.Count;
		}

		public static int GetVietPhraseDictionaryCount()
		{
			return onlyVietPhraseDictionary.Count;
		}

		public static int GetNameDictionaryCount(bool isNameChinh)
		{
			if (isNameChinh)
			{
				return onlyNameChinhDictionary.Count;
			}
			return onlyNamePhuDictionary.Count;
		}

		public static int GetPhienAmDictionaryCount()
		{
			return hanVietDictionary.Count;
		}

		public static int GetDanhTuDictionaryCount()
		{
			return danhTuDictionary.Count;
		}

		public static int GetHoNguoiDictionaryCount()
		{
			return hoNguoiDictionary.Count;
		}

		public static int GetHauTuDictionaryCount()
		{
			return hauTuDictionary.Count;
		}

		private static void updateHistoryLogInCache(string key, string action, ref DataSet dictionaryHistoryDataSet)
		{
			string name = "DictionaryHistory";
			DataRow dataRow = dictionaryHistoryDataSet.Tables[name].Rows.Find(key);
			if (dataRow == null)
			{
				dictionaryHistoryDataSet.Tables[name].Rows.Add(key, action, Environment.GetEnvironmentVariable("USERNAME"), DateTime.Now);
			}
			else
			{
				dataRow[1] = action;
				dataRow[2] = Environment.GetEnvironmentVariable("USERNAME");
				dataRow[3] = DateTime.Now;
			}
		}

		private static void writeVietPhraseHistoryLog(string key, string action)
		{
			updateHistoryLogInCache(key, action, ref onlyVietPhraseDictionaryHistoryDataSet);
			WriteHistoryLog(key, action, DictionaryConfigurationHelper.GetVietPhraseDictionaryHistoryPath());
		}

		private static void writeNamesHistoryLog(string key, string action, bool isNameChinh)
		{
			DataSet dictionaryHistoryDataSet = (isNameChinh ? onlyNameDictionaryHistoryDataSet : onlyNamePhuDictionaryHistoryDataSet);
			updateHistoryLogInCache(key, action, ref dictionaryHistoryDataSet);
			WriteHistoryLog(key, action, isNameChinh ? DictionaryConfigurationHelper.GetNamesDictionaryHistoryPath() : DictionaryConfigurationHelper.GetNamesPhuDictionaryHistoryPath());
		}

		private static void writePhienAmHistoryLog(string key, string action)
		{
			updateHistoryLogInCache(key, action, ref hanVietDictionaryHistoryDataSet);
			WriteHistoryLog(key, action, DictionaryConfigurationHelper.GetChinesePhienAmWordsDictionaryHistoryPath());
		}

		public static string GetVietPhraseHistoryLogRecord(string key)
		{
			return getDictionaryHistoryLogRecordInCache(key, onlyVietPhraseDictionaryHistoryDataSet);
		}

		public static string GetNameHistoryLogRecord(string key, bool isNameChinh)
		{
			return getDictionaryHistoryLogRecordInCache(key, isNameChinh ? onlyNameDictionaryHistoryDataSet : onlyNamePhuDictionaryHistoryDataSet);
		}

		public static string GetPhienAmHistoryLogRecord(string key)
		{
			return getDictionaryHistoryLogRecordInCache(key, hanVietDictionaryHistoryDataSet);
		}

		private static string getDictionaryHistoryLogRecordInCache(string key, DataSet dictionaryHistoryDataSet)
		{
			string name = "DictionaryHistory";
			DataRow dataRow = dictionaryHistoryDataSet.Tables[name].Rows.Find(key);
			if (dataRow != null)
			{
				return string.Format("Đã được <{0}> bởi <{1}> vào <{2}>.", dataRow[1], dataRow[2], ((DateTime)dataRow[3]).ToString("yyyy-MM-dd HH:mm:ss"));
			}
			return "";
		}

		public static void CompressPhienAmDictionaryHistory()
		{
			CompressDictionaryHistory(hanVietDictionaryHistoryDataSet, DictionaryConfigurationHelper.GetChinesePhienAmWordsDictionaryHistoryPath());
		}

		public static void CompressOnlyVietPhraseDictionaryHistory()
		{
			CompressDictionaryHistory(onlyVietPhraseDictionaryHistoryDataSet, DictionaryConfigurationHelper.GetVietPhraseDictionaryHistoryPath());
		}

		public static void CompressOnlyNameDictionaryHistory(bool isNameChinh)
		{
			CompressDictionaryHistory(isNameChinh ? onlyNameDictionaryHistoryDataSet : onlyNamePhuDictionaryHistoryDataSet, isNameChinh ? DictionaryConfigurationHelper.GetNamesDictionaryHistoryPath() : DictionaryConfigurationHelper.GetNamesPhuDictionaryHistoryPath());
		}

		private static void CompressDictionaryHistory(DataSet dictionaryHistoryDataSet, string dictionaryHistoryFilePath)
		{
			string name = "DictionaryHistory";
			string text = dictionaryHistoryFilePath + "." + DateTime.Now.Ticks;
			if (File.Exists(dictionaryHistoryFilePath))
			{
				File.Copy(dictionaryHistoryFilePath, text, overwrite: true);
			}
			using TextWriter textWriter = new StreamWriter(dictionaryHistoryFilePath, append: false, Encoding.UTF8);
			try
			{
				textWriter.WriteLine("Entry\tAction\tUser Name\tUpdated Date");
				foreach (DataRow row in dictionaryHistoryDataSet.Tables[name].Rows)
				{
					textWriter.WriteLine(string.Format("{0}\t{1}\t{2}\t{3}", row[0], row[1], row[2], ((DateTime)row[3]).ToString("yyyy-MM-dd HH:mm:ss")));
				}
			}
			catch (Exception ex)
			{
				try
				{
					textWriter.Close();
				}
				catch
				{
				}
				if (File.Exists(dictionaryHistoryFilePath))
				{
					try
					{
						File.Copy(text, dictionaryHistoryFilePath, overwrite: true);
					}
					catch
					{
					}
				}
				throw ex;
			}
			finally
			{
				File.Delete(text);
			}
		}

		public static void WriteHistoryLog(string key, string action, string logPath)
		{
			if (!File.Exists(logPath))
			{
				File.AppendAllText(logPath, "Entry\tAction\tUser Name\tUpdated Date\r\n", Encoding.UTF8);
			}
			File.AppendAllText(logPath, key + "\t" + action + "\t" + Environment.GetEnvironmentVariable("USERNAME") + "\t" + DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss") + "\r\n", Encoding.UTF8);
		}

		public static void CreateHistoryLog(string key, string action, ref StringBuilder historyLogs)
		{
			historyLogs.AppendLine(key + "\t" + action + "\t" + Environment.GetEnvironmentVariable("USERNAME") + "\t" + DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss"));
		}

		public static void WriteHistoryLog(string historyLogs, string logPath)
		{
			if (!File.Exists(logPath))
			{
				File.AppendAllText(logPath, "Entry\tAction\tUser Name\tUpdated Date\r\n", Encoding.UTF8);
			}
			File.AppendAllText(logPath, historyLogs, Encoding.UTF8);
		}

		private static string removeIgnoredChinesePhrases(string standardizedChinese)
		{
			if (string.IsNullOrEmpty(standardizedChinese))
			{
				return string.Empty;
			}
			string text = standardizedChinese;
			foreach (string ignoredChinesePhrase in ignoredChinesePhraseList)
			{
				text = text.Replace(ignoredChinesePhrase, string.Empty);
			}
			return text.Replace("\t\n\n", string.Empty);
		}

		private static int containsLuatNhan(string chinese, out string luatNhan, out int matchedLength, out string valueN)
		{
			luatNhan = string.Empty;
			matchedLength = -1;
			valueN = string.Empty;
			int num = int.MaxValue;
			Dictionary<string, string> dictionary = luatNhanNList;
			if (dictionary != null && dictionary.Count > 0)
			{
				string matchedLuatNhan;
				int matchedLength2;
				string valueN2;
				int num2 = matchLuatNhanWithN(chinese, out matchedLuatNhan, out matchedLength2, out valueN2);
				if (num2 >= 0 && num2 < num)
				{
					num = num2;
					luatNhan = matchedLuatNhan;
					matchedLength = matchedLength2;
					valueN = valueN2;
				}
			}
			for (int i = 0; i < Math.Min(chinese.Length, num); i++)
			{
				FindHoHauTuPhrase(chinese, i, out var bestHTLength);
				if (bestHTLength > 0)
				{
					num = i;
					luatNhan = "{h}{t}";
					matchedLength = bestHTLength;
					valueN = string.Empty;
					break;
				}
			}
			if (num > 0 && chinese.Any((char c) => NumberChars.Contains(c)) && matchLuatNhanS(chinese, out var luatNhan2, out var matchedLength3, out var matchIndex, out var valueN3) && matchIndex < num)
			{
				num = matchIndex;
				luatNhan = luatNhan2;
				matchedLength = matchedLength3;
				valueN = valueN3;
			}
			if (num == int.MaxValue)
			{
				return -1;
			}
			return num;
		}

		private static bool matchLuatNhanS(string chinese, out string luatNhan, out int matchedLength, out int matchIndex, out string valueN)
		{
			luatNhan = string.Empty;
			valueN = string.Empty;
			matchedLength = -1;
			matchIndex = -1;
			foreach (string key in luatNhanSList.Keys)
			{
				if (!luatNhanSCache.TryGetValue(key, out var value))
				{
					continue;
				}
				Match match = value.Match(chinese);
				if (match.Success && match.Groups.Count > 1 && !onlyVietPhraseDictionary.ContainsKey(match.Value))
				{
					string value2 = match.Groups[1].Value;
					if (!string.IsNullOrWhiteSpace(value2))
					{
						valueN = value2;
						luatNhan = key;
						matchedLength = match.Length;
						matchIndex = match.Index;
						return true;
					}
				}
			}
			return false;
		}

		private static int matchLuatNhanWithN(string chinese, out string matchedLuatNhan, out int matchedLength, out string valueN)
		{
			matchedLuatNhan = string.Empty;
			valueN = string.Empty;
			matchedLength = -1;
			Dictionary<string, Regex> dictionary = luatNhanNCache;
			foreach (KeyValuePair<string, string> item in luatNhanNList.OrderByDescending((KeyValuePair<string, string> kv) => GetNormalizedLength(kv.Key)))
			{
				if (!dictionary.TryGetValue(item.Key, out var value))
				{
					continue;
				}
				Match match = value.Match(chinese);
				while (match.Success)
				{
					if (match.Groups.Count > 1)
					{
						string key = item.Key;
						string value2 = match.Groups[1].Value;
						string value5;
						if (key.EndsWith("{n}"))
						{
							for (int num = value2.Length; num > 0; num--)
							{
								string key2 = value2.Substring(0, num);
								if (dictionaryN.TryGetValue(key2, out var value3))
								{
									valueN = value3;
									matchedLuatNhan = key;
									matchedLength = match.Length - (value2.Length - num);
									return match.Index;
								}
							}
						}
						else if (key.StartsWith("{n}"))
						{
							for (int i = 0; i < value2.Length; i++)
							{
								string key3 = value2.Substring(i);
								if (dictionaryN.TryGetValue(key3, out var value4))
								{
									valueN = value4;
									matchedLuatNhan = key;
									matchedLength = match.Length - i;
									return match.Index + i;
								}
							}
						}
						else if (dictionaryN.TryGetValue(value2, out value5))
						{
							valueN = value5;
							matchedLuatNhan = key;
							matchedLength = match.Length;
							return match.Index;
						}
					}
					match = match.NextMatch();
				}
			}
			return -1;
		}

		public static string ChineseToLuatNhanOneMeaning(string chinese, string luatNhan, string valueN)
		{
			if (luatNhan.Contains("{n}"))
			{
				if (string.IsNullOrWhiteSpace(valueN))
				{
					return null;
				}
				return luatNhanNList[luatNhan].Replace("{n}", valueN.Trim());
			}
			if (luatNhan == "{s}两")
			{
				return TransLuatNhan.TranslateSLuongRule(chinese);
			}
			if (luatNhan == "百分[之]?{s}")
			{
				if (string.IsNullOrWhiteSpace(valueN))
				{
					return null;
				}
				string newValue = TransLuatNhan.ConvertChineseDecimalToString(valueN);
				return luatNhanSList[luatNhan].Replace("{s}", newValue);
			}
			if (luatNhan.Contains("{s}"))
			{
				if (luatNhan.Contains("余") || luatNhan.Contains("多"))
				{
					int num = chinese.IndexOf('余');
					if (num == -1)
					{
						num = chinese.IndexOf('多');
					}
					if (num != -1)
					{
						try
						{
							string text = chinese.Remove(num, 1);
							long num2 = TransLuatNhan.ConvertChineseNumberToLong(text);
							bool flag = text.Trim() == "0";
							if (num2 != 0 || flag)
							{
								string newValue2 = TransLuatNhan.NumberToVietnameseText(num2);
								return luatNhanSList[luatNhan].Replace("{s}", newValue2);
							}
						}
						catch
						{
						}
					}
				}
				int count = Regex.Matches(luatNhan, "\\{s\\}").Count;
				if (count == 0)
				{
					return null;
				}
				string newValue3 = "((?:(?:\\d+(?:[.,]\\d+)?|[零一二三四五六七八九十百千万亿两〇]+)\\s*)+)";
				bool num3 = chinese.Any(char.IsDigit);
				string text2 = luatNhan;
				if (text2.Contains("("))
				{
					text2 = text2.Replace("(", "(?:");
				}
				string pattern;
				if (num3)
				{
					string text3 = text2.Replace("{s}", " {s} ");
					text3 = Regex.Replace(text3.Trim(), "\\s+", "\\s*");
					pattern = "^" + text3.Replace("{s}", newValue3) + "$";
				}
				else
				{
					pattern = "^" + text2.Replace("{s}", newValue3) + "$";
				}
				Match match = Regex.Match(chinese.Trim(), pattern);
				if ((!match.Success || match.Groups.Count - 1 != count) && count == 1)
				{
					return null;
				}
				try
				{
					string text4 = luatNhanSList[luatNhan];
					string text6;
					if (count == 1)
					{
						string text5 = match.Groups[1].Value.Trim();
						if (string.IsNullOrWhiteSpace(text5))
						{
							return null;
						}
						string newValue4;
						string result;
						string result2;
						int value;
						int value2;
						if (Enumerable.Contains(text5, '.') || Enumerable.Contains(text5, ',') || (text5.Length > 1 && text5.StartsWith("0")))
						{
							newValue4 = text5;
						}
						else if (TransLuatNhan.TryConvertVietnameseRangeNumber(text5, out result))
						{
							newValue4 = result;
						}
						else if (TransLuatNhan.TryConvertPostfixedRangeNumber(text5, out result2))
						{
							newValue4 = result2;
						}
						else if (text5.Length == 2 && TransLuatNhan.chineseNumberMap.TryGetValue(text5[0], out value) && TransLuatNhan.chineseNumberMap.TryGetValue(text5[1], out value2))
						{
							newValue4 = $"{value}-{value2}";
						}
						else
						{
							bool num4 = Regex.IsMatch(text5, "^(\\d+|[零一二三四五六七八九十百千万亿两〇]+)$") && Regex.IsMatch(text4, "(năm|chương)\\s*\\{s\\}", RegexOptions.IgnoreCase);
							long number = TransLuatNhan.ConvertChineseNumberToLong(text5);
							newValue4 = (num4 ? number.ToString() : TransLuatNhan.NumberToVietnameseText(number));
						}
						text6 = text4.Replace("{s}", newValue4);
					}
					else
					{
						List<string> list = new List<string>();
						string text7 = text4;
						for (int i = 1; i <= count; i++)
						{
							string text8 = match.Groups[i].Value.Trim();
							if (string.IsNullOrWhiteSpace(text8))
							{
								return null;
							}
							bool num5 = Regex.IsMatch(text8, "^(\\d+|[零一二三四五六七八九十百千万亿两〇]+)$") && Regex.IsMatch(text4, $"(năm|chương)\\s*\\{{{i}\\}}", RegexOptions.IgnoreCase);
							long number2 = TransLuatNhan.ConvertChineseNumberToLong(text8);
							string item = (num5 ? number2.ToString() : TransLuatNhan.NumberToVietnameseText(number2));
							list.Add(item);
						}
						if (list.Count != count)
						{
							return null;
						}
						for (int j = 0; j < list.Count; j++)
						{
							text7 = text7.Replace("{" + (j + 1) + "}", list[j]);
						}
						text6 = text7;
					}
					Match match2 = Regex.Match(text6, "^ngày\\s+(\\d+)", RegexOptions.IgnoreCase);
					if (match2.Success && int.TryParse(match2.Groups[1].Value, out var result3) && result3 > 0 && result3 < 10)
					{
						text6 = Regex.Replace(text6, "^ngày", "mùng", RegexOptions.IgnoreCase);
					}
					return text6;
				}
				catch
				{
					return null;
				}
			}
			if (luatNhan == "{h}{t}")
			{
				for (int k = 1; k < chinese.Length; k++)
				{
					string key = chinese.Substring(0, k);
					string key2 = chinese.Substring(k);
					if (hoNguoiDictionary.TryGetValue(key, out var value3) && hauTuDictionary.TryGetValue(key2, out var value4))
					{
						return value3.Trim() + " " + value4.Trim();
					}
				}
				return null;
			}
			return null;
		}

		static TranslatorEngine()
		{
			luatNhanSCache = new Dictionary<string, Regex>();
			luatNhanNCache = new Dictionary<string, Regex>();
			hoHauTuCache = new HashSet<string>();
			NumberChars = new HashSet<char>
			{
				'零', '一', '二', '三', '四', '五', '六', '七', '八', '九',
				'十', '百', '千', '万', '亿', '两', '〇', '0', '1', '2',
				'3', '4', '5', '6', '7', '8', '9'
			};
			dictionaryDirty = true;
			isLoading = false;
			resultHanViet = new StringBuilder();
			danhTuDictionary = new Dictionary<string, string>();
			hoNguoiDictionary = new Dictionary<string, string>();
			hauTuDictionary = new Dictionary<string, string>();
			vietPhraseDictionary = new Dictionary<string, string>();
			onlyNamePhuDictionary = new Dictionary<string, string>();
			onlyVietPhraseDictionary = new Dictionary<string, string>();
			onlyNameDictionary = new Dictionary<string, string>();
			dictionaryDirty = true;
			hanVietDictionary = new Dictionary<string, string>();
			lacVietDictionary = new Dictionary<string, string>();
			vietPhraseOneMeaningDictionary = new Dictionary<string, string>();
			onlyNameOneMeaningDictionary = new Dictionary<string, string>();
			onlyNameChinhDictionary = new Dictionary<string, string>();
			luatNhanDictionary = new Dictionary<string, string>();
			pronounDictionary = new Dictionary<string, string>();
			onlyVietPhraseDictionaryHistoryDataSet = new DataSet();
			onlyNameDictionaryHistoryDataSet = new DataSet();
			onlyNamePhuDictionaryHistoryDataSet = new DataSet();
			hanVietDictionaryHistoryDataSet = new DataSet();
			ignoredChinesePhraseList = new List<string>();
			ignoredChinesePhraseForBrowserList = new List<string>();
			lockObject = new object();
			NULL_STRING = Convert.ToChar(0).ToString();
			LastTranslatedWord_HanViet = "";
			LastTranslatedWord_VietPhrase = "";
			LastTranslatedWord_VietPhraseOneMeaning = "";
			changeCounters = new Dictionary<DCType, int>
			{
				{
					DCType.HoNguoi,
					1
				},
				{
					DCType.Pronoun,
					1
				},
				{
					DCType.HauTu,
					1
				},
				{
					DCType.HanViet,
					1
				},
				{
					DCType.LacViet,
					1
				},
				{
					DCType.DanhTu,
					1
				}
			};
		}

		public static void WriteDebugLog(string message)
		{
			string path = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "debug_log.txt");
			string text = message ?? "";
			File.AppendAllText(path, text + Environment.NewLine);
		}
	}
}
namespace org.mozilla.intl.chardet
{
	public class Big5Statistics : nsEUCStatistics
	{
		private static float[] m_FirstByteFreq;

		private static float m_FirstByteStdDev;

		private static float m_FirstByteMean;

		private static float m_FirstByteWeight;

		private static float[] m_SecondByteFreq;

		private static float m_SecondByteStdDev;

		private static float m_SecondByteMean;

		private static float m_SecondByteWeight;

		public override float[] mFirstByteFreq()
		{
			return m_FirstByteFreq;
		}

		public override float mFirstByteStdDev()
		{
			return m_FirstByteStdDev;
		}

		public override float mFirstByteMean()
		{
			return m_FirstByteMean;
		}

		public override float mFirstByteWeight()
		{
			return m_FirstByteWeight;
		}

		public override float[] mSecondByteFreq()
		{
			return m_SecondByteFreq;
		}

		public override float mSecondByteStdDev()
		{
			return m_SecondByteStdDev;
		}

		public override float mSecondByteMean()
		{
			return m_SecondByteMean;
		}

		public override float mSecondByteWeight()
		{
			return m_SecondByteWeight;
		}

		public Big5Statistics()
		{
			m_FirstByteFreq = new float[94]
			{
				0f, 0f, 0f, 0.114427f, 0.061058f, 0.075598f, 0.048386f, 0.063966f, 0.027094f, 0.095787f,
				0.029525f, 0.031331f, 0.036915f, 0.021805f, 0.019349f, 0.037496f, 0.018068f, 0.01276f, 0.030053f, 0.017339f,
				0.016731f, 0.019501f, 0.01124f, 0.032973f, 0.016658f, 0.015872f, 0.021458f, 0.012378f, 0.017003f, 0.020802f,
				0.012454f, 0.009239f, 0.012829f, 0.007922f, 0.010079f, 0.009815f, 0.010104f, 0f, 0f, 0f,
				5.3E-05f, 3.5E-05f, 0.000105f, 3.1E-05f, 8.8E-05f, 2.7E-05f, 2.7E-05f, 2.6E-05f, 3.5E-05f, 2.4E-05f,
				3.4E-05f, 0.000375f, 2.5E-05f, 2.8E-05f, 2E-05f, 2.4E-05f, 2.8E-05f, 3.1E-05f, 5.9E-05f, 4E-05f,
				3E-05f, 7.9E-05f, 3.7E-05f, 4E-05f, 2.3E-05f, 3E-05f, 2.7E-05f, 6.4E-05f, 2E-05f, 2.7E-05f,
				2.5E-05f, 7.4E-05f, 1.9E-05f, 2.3E-05f, 2.1E-05f, 1.8E-05f, 1.7E-05f, 3.5E-05f, 2.1E-05f, 1.9E-05f,
				2.5E-05f, 1.7E-05f, 3.7E-05f, 1.8E-05f, 1.8E-05f, 1.9E-05f, 2.2E-05f, 3.3E-05f, 3.2E-05f, 0f,
				0f, 0f, 0f, 0f
			};
			m_FirstByteStdDev = 0.020606f;
			m_FirstByteMean = 0.010638f;
			m_FirstByteWeight = 0.675261f;
			m_SecondByteFreq = new float[94]
			{
				0.020256f, 0.003293f, 0.045811f, 0.01665f, 0.007066f, 0.004146f, 0.009229f, 0.007333f, 0.003296f, 0.005239f,
				0.008282f, 0.003791f, 0.006116f, 0.003536f, 0.004024f, 0.016654f, 0.009334f, 0.005429f, 0.033392f, 0.006121f,
				0.008983f, 0.002801f, 0.004221f, 0.010357f, 0.014695f, 0.077937f, 0.006314f, 0.00402f, 0.007331f, 0.00715f,
				0.005341f, 0.009195f, 0.00535f, 0.005698f, 0.004472f, 0.007242f, 0.004039f, 0.011154f, 0.016184f, 0.004741f,
				0.012814f, 0.007679f, 0.008045f, 0.016631f, 0.009451f, 0.016487f, 0.007287f, 0.012688f, 0.017421f, 0.013205f,
				0.03148f, 0.003404f, 0.009149f, 0.008921f, 0.007514f, 0.008683f, 0.008203f, 0.031403f, 0.011733f, 0.015617f,
				0.015306f, 0.004004f, 0.010899f, 0.009961f, 0.008388f, 0.01092f, 0.003925f, 0.008585f, 0.009108f, 0.015546f,
				0.004659f, 0.006934f, 0.007023f, 0.020252f, 0.005387f, 0.024704f, 0.006963f, 0.002625f, 0.009512f, 0.002971f,
				0.008233f, 0.01f, 0.011973f, 0.010553f, 0.005945f, 0.006349f, 0.009401f, 0.008577f, 0.008186f, 0.008159f,
				0.005033f, 0.008714f, 0.010614f, 0.006554f
			};
			m_SecondByteStdDev = 0.009909f;
			m_SecondByteMean = 0.010638f;
			m_SecondByteWeight = 0.324739f;
		}
	}
	public class EUCJPStatistics : nsEUCStatistics
	{
		private static float[] m_FirstByteFreq;

		private static float m_FirstByteStdDev;

		private static float m_FirstByteMean;

		private static float m_FirstByteWeight;

		private static float[] m_SecondByteFreq;

		private static float m_SecondByteStdDev;

		private static float m_SecondByteMean;

		private static float m_SecondByteWeight;

		public override float[] mFirstByteFreq()
		{
			return m_FirstByteFreq;
		}

		public override float mFirstByteStdDev()
		{
			return m_FirstByteStdDev;
		}

		public override float mFirstByteMean()
		{
			return m_FirstByteMean;
		}

		public override float mFirstByteWeight()
		{
			return m_FirstByteWeight;
		}

		public override float[] mSecondByteFreq()
		{
			return m_SecondByteFreq;
		}

		public override float mSecondByteStdDev()
		{
			return m_SecondByteStdDev;
		}

		public override float mSecondByteMean()
		{
			return m_SecondByteMean;
		}

		public override float mSecondByteWeight()
		{
			return m_SecondByteWeight;
		}

		public EUCJPStatistics()
		{
			m_FirstByteFreq = new float[94]
			{
				0.364808f, 0f, 0f, 0.145325f, 0.304891f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0.001835f, 0.010771f, 0.006462f, 0.001157f, 0.002114f,
				0.003231f, 0.001356f, 0.00742f, 0.004189f, 0.003231f, 0.003032f, 0.03319f, 0.006303f, 0.006064f, 0.009973f,
				0.002354f, 0.00367f, 0.009135f, 0.001675f, 0.002792f, 0.002194f, 0.01472f, 0.011928f, 0.000878f, 0.013124f,
				0.001077f, 0.009295f, 0.003471f, 0.002872f, 0.002433f, 0.000957f, 0.001636f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 8E-05f, 0.000279f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 8E-05f, 0f
			};
			m_FirstByteStdDev = 0.050407f;
			m_FirstByteMean = 0.010638f;
			m_FirstByteWeight = 0.640871f;
			m_SecondByteFreq = new float[94]
			{
				0.002473f, 0.039134f, 0.152745f, 0.009694f, 0.000359f, 0.02218f, 0.000758f, 0.004308f, 0.00016f, 0.002513f,
				0.003072f, 0.001316f, 0.00383f, 0.001037f, 0.00359f, 0.000957f, 0.00016f, 0.000239f, 0.006462f, 0.001596f,
				0.031554f, 0.001316f, 0.002194f, 0.016555f, 0.003271f, 0.000678f, 0.000598f, 0.206438f, 0.000718f, 0.001077f,
				0.00371f, 0.001356f, 0.001356f, 0.000439f, 0.004388f, 0.005704f, 0.000878f, 0.010172f, 0.007061f, 0.01468f,
				0.000638f, 0.02573f, 0.002792f, 0.000718f, 0.001795f, 0.091551f, 0.000758f, 0.003909f, 0.000558f, 0.031195f,
				0.007061f, 0.001316f, 0.022579f, 0.006981f, 0.00726f, 0.001117f, 0.000239f, 0.012127f, 0.000878f, 0.00379f,
				0.001077f, 0.000758f, 0.002114f, 0.002234f, 0.000678f, 0.002992f, 0.003311f, 0.023416f, 0.001237f, 0.002753f,
				0.005146f, 0.002194f, 0.007021f, 0.008497f, 0.013763f, 0.011768f, 0.006303f, 0.001915f, 0.000638f, 0.008776f,
				0.000918f, 0.003431f, 0.057603f, 0.000439f, 0.000439f, 0.000758f, 0.002872f, 0.001675f, 0.01105f, 0f,
				0.000279f, 0.012127f, 0.000718f, 0.00738f
			};
			m_SecondByteStdDev = 0.028247f;
			m_SecondByteMean = 0.010638f;
			m_SecondByteWeight = 0.359129f;
		}
	}
	public class EUCKRStatistics : nsEUCStatistics
	{
		private static float[] m_FirstByteFreq;

		private static float m_FirstByteStdDev;

		private static float m_FirstByteMean;

		private static float m_FirstByteWeight;

		private static float[] m_SecondByteFreq;

		private static float m_SecondByteStdDev;

		private static float m_SecondByteMean;

		private static float m_SecondByteWeight;

		public override float[] mFirstByteFreq()
		{
			return m_FirstByteFreq;
		}

		public override float mFirstByteStdDev()
		{
			return m_FirstByteStdDev;
		}

		public override float mFirstByteMean()
		{
			return m_FirstByteMean;
		}

		public override float mFirstByteWeight()
		{
			return m_FirstByteWeight;
		}

		public override float[] mSecondByteFreq()
		{
			return m_SecondByteFreq;
		}

		public override float mSecondByteStdDev()
		{
			return m_SecondByteStdDev;
		}

		public override float mSecondByteMean()
		{
			return m_SecondByteMean;
		}

		public override float mSecondByteWeight()
		{
			return m_SecondByteWeight;
		}

		public EUCKRStatistics()
		{
			m_FirstByteFreq = new float[94]
			{
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0.000412f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0.057502f, 0.033182f, 0.002267f, 0.016076f, 0.014633f,
				0.032976f, 0.004122f, 0.011336f, 0.058533f, 0.024526f, 0.025969f, 0.054411f, 0.01958f, 0.063273f, 0.113974f,
				0.029885f, 0.150041f, 0.059151f, 0.002679f, 0.009893f, 0.014839f, 0.026381f, 0.015045f, 0.069456f, 0.08986f,
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f
			};
			m_FirstByteStdDev = 0.025593f;
			m_FirstByteMean = 0.010638f;
			m_FirstByteWeight = 0.647437f;
			m_SecondByteFreq = new float[94]
			{
				0.016694f, 0f, 0.012778f, 0.030091f, 0.002679f, 0.006595f, 0.001855f, 0.000824f, 0.005977f, 0.00474f,
				0.003092f, 0.000824f, 0.01958f, 0.037304f, 0.008244f, 0.014633f, 0.001031f, 0f, 0.003298f, 0.002061f,
				0.006183f, 0.005977f, 0.000824f, 0.021847f, 0.014839f, 0.052968f, 0.017312f, 0.007626f, 0.000412f, 0.000824f,
				0.011129f, 0f, 0.000412f, 0.001649f, 0.005977f, 0.065746f, 0.020198f, 0.021434f, 0.014633f, 0.004122f,
				0.001649f, 0.000824f, 0.000824f, 0.051937f, 0.01958f, 0.023289f, 0.026381f, 0.040396f, 0.009068f, 0.001443f,
				0.00371f, 0.00742f, 0.001443f, 0.01319f, 0.002885f, 0.000412f, 0.003298f, 0.025969f, 0.000412f, 0.000412f,
				0.006183f, 0.003298f, 0.066983f, 0.002679f, 0.002267f, 0.011129f, 0.000412f, 0.010099f, 0.015251f, 0.007626f,
				0.043899f, 0.00371f, 0.002679f, 0.001443f, 0.010923f, 0.002885f, 0.009068f, 0.019992f, 0.000412f, 0.00845f,
				0.005153f, 0f, 0.010099f, 0f, 0.001649f, 0.01216f, 0.011542f, 0.006595f, 0.001855f, 0.010923f,
				0.000412f, 0.023702f, 0.00371f, 0.001855f
			};
			m_SecondByteStdDev = 0.013937f;
			m_SecondByteMean = 0.010638f;
			m_SecondByteWeight = 0.352563f;
		}
	}
	public class EUCTWStatistics : nsEUCStatistics
	{
		private static float[] m_FirstByteFreq;

		private static float m_FirstByteStdDev;

		private static float m_FirstByteMean;

		private static float m_FirstByteWeight;

		private static float[] m_SecondByteFreq;

		private static float m_SecondByteStdDev;

		private static float m_SecondByteMean;

		private static float m_SecondByteWeight;

		public override float[] mFirstByteFreq()
		{
			return m_FirstByteFreq;
		}

		public override float mFirstByteStdDev()
		{
			return m_FirstByteStdDev;
		}

		public override float mFirstByteMean()
		{
			return m_FirstByteMean;
		}

		public override float mFirstByteWeight()
		{
			return m_FirstByteWeight;
		}

		public override float[] mSecondByteFreq()
		{
			return m_SecondByteFreq;
		}

		public override float mSecondByteStdDev()
		{
			return m_SecondByteStdDev;
		}

		public override float mSecondByteMean()
		{
			return m_SecondByteMean;
		}

		public override float mSecondByteWeight()
		{
			return m_SecondByteWeight;
		}

		public EUCTWStatistics()
		{
			m_FirstByteFreq = new float[94]
			{
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0.119286f, 0.052233f, 0.044126f, 0.052494f, 0.045906f,
				0.019038f, 0.032465f, 0.026252f, 0.025502f, 0.015963f, 0.052493f, 0.019256f, 0.015137f, 0.031782f, 0.01737f,
				0.018494f, 0.015575f, 0.016621f, 0.007444f, 0.011642f, 0.013916f, 0.019159f, 0.016445f, 0.007851f, 0.011079f,
				0.022842f, 0.015513f, 0.010033f, 0.00995f, 0.010347f, 0.013103f, 0.015371f, 0.012502f, 0.007436f, 0.018253f,
				0.014134f, 0.008907f, 0.005411f, 0.00957f, 0.013598f, 0.006092f, 0.007409f, 0.008432f, 0.005816f, 0.009349f,
				0.005472f, 0.00717f, 0.00742f, 0.003681f, 0.007523f, 0.00461f, 0.006154f, 0.003348f, 0.005074f, 0.005922f,
				0.005254f, 0.004682f, 0.002093f, 0f
			};
			m_FirstByteStdDev = 0.016681f;
			m_FirstByteMean = 0.010638f;
			m_FirstByteWeight = 0.715599f;
			m_SecondByteFreq = new float[94]
			{
				0.028933f, 0.011371f, 0.011053f, 0.007232f, 0.010192f, 0.004093f, 0.015043f, 0.011752f, 0.022387f, 0.00841f,
				0.012448f, 0.007473f, 0.003594f, 0.007139f, 0.018912f, 0.006083f, 0.003302f, 0.010215f, 0.008791f, 0.024236f,
				0.014107f, 0.014108f, 0.010303f, 0.009728f, 0.007877f, 0.009719f, 0.007952f, 0.021028f, 0.005764f, 0.009341f,
				0.006591f, 0.012517f, 0.005921f, 0.008982f, 0.008771f, 0.012802f, 0.005926f, 0.008342f, 0.003086f, 0.006843f,
				0.007576f, 0.004734f, 0.016404f, 0.008803f, 0.008071f, 0.005349f, 0.008566f, 0.01084f, 0.015401f, 0.031904f,
				0.00867f, 0.011479f, 0.010936f, 0.007617f, 0.008995f, 0.008114f, 0.008658f, 0.005934f, 0.010452f, 0.009142f,
				0.004519f, 0.008339f, 0.007476f, 0.007027f, 0.006025f, 0.021804f, 0.024248f, 0.015895f, 0.003768f, 0.010171f,
				0.010007f, 0.010178f, 0.008316f, 0.006832f, 0.006364f, 0.009141f, 0.009148f, 0.012081f, 0.011914f, 0.004464f,
				0.014257f, 0.006907f, 0.011292f, 0.018622f, 0.008149f, 0.004636f, 0.006612f, 0.013478f, 0.012614f, 0.005186f,
				0.048285f, 0.006816f, 0.006743f, 0.008671f
			};
			m_SecondByteStdDev = 0.00663f;
			m_SecondByteMean = 0.010638f;
			m_SecondByteWeight = 0.284401f;
		}
	}
	public class GB2312Statistics : nsEUCStatistics
	{
		private static float[] m_FirstByteFreq;

		private static float m_FirstByteStdDev;

		private static float m_FirstByteMean;

		private static float m_FirstByteWeight;

		private static float[] m_SecondByteFreq;

		private static float m_SecondByteStdDev;

		private static float m_SecondByteMean;

		private static float m_SecondByteWeight;

		public override float[] mFirstByteFreq()
		{
			return m_FirstByteFreq;
		}

		public override float mFirstByteStdDev()
		{
			return m_FirstByteStdDev;
		}

		public override float mFirstByteMean()
		{
			return m_FirstByteMean;
		}

		public override float mFirstByteWeight()
		{
			return m_FirstByteWeight;
		}

		public override float[] mSecondByteFreq()
		{
			return m_SecondByteFreq;
		}

		public override float mSecondByteStdDev()
		{
			return m_SecondByteStdDev;
		}

		public override float mSecondByteMean()
		{
			return m_SecondByteMean;
		}

		public override float mSecondByteWeight()
		{
			return m_SecondByteWeight;
		}

		public GB2312Statistics()
		{
			m_FirstByteFreq = new float[94]
			{
				0.011628f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0.011628f, 0.012403f, 0.009302f, 0.003876f, 0.017829f,
				0.037209f, 0.008527f, 0.010078f, 0.01938f, 0.054264f, 0.010078f, 0.041085f, 0.02093f, 0.018605f, 0.010078f,
				0.013178f, 0.016279f, 0.006202f, 0.009302f, 0.017054f, 0.011628f, 0.008527f, 0.004651f, 0.006202f, 0.017829f,
				0.024806f, 0.020155f, 0.013953f, 0.032558f, 0.035659f, 0.068217f, 0.010853f, 0.036434f, 0.117054f, 0.027907f,
				0.100775f, 0.010078f, 0.017829f, 0.062016f, 0.012403f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0.00155f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f,
				0f, 0f, 0f, 0f
			};
			m_FirstByteStdDev = 0.020081f;
			m_FirstByteMean = 0.010638f;
			m_FirstByteWeight = 0.586533f;
			m_SecondByteFreq = new float[94]
			{
				0.006202f, 0.031008f, 0.005426f, 0.003101f, 0.00155f, 0.003101f, 0.082171f, 0.014729f, 0.006977f, 0.00155f,
				0.013953f, 0f, 0.013953f, 0.010078f, 0.008527f, 0.006977f, 0.004651f, 0.003101f, 0.003101f, 0.003101f,
				0.008527f, 0.003101f, 0.005426f, 0.005426f, 0.005426f, 0.003101f, 0.00155f, 0.006202f, 0.014729f, 0.010853f,
				0f, 0.011628f, 0f, 0.031783f, 0.013953f, 0.030233f, 0.039535f, 0.008527f, 0.015504f, 0f,
				0.003101f, 0.008527f, 0.016279f, 0.005426f, 0.00155f, 0.013953f, 0.013953f, 0.044961f, 0.003101f, 0.004651f,
				0.006977f, 0.00155f, 0.005426f, 0.012403f, 0.00155f, 0.015504f, 0f, 0.006202f, 0.00155f, 0f,
				0.007752f, 0.006977f, 0.00155f, 0.009302f, 0.011628f, 0.004651f, 0.010853f, 0.012403f, 0.017829f, 0.005426f,
				0.024806f, 0f, 0.006202f, 0f, 0.082171f, 0.015504f, 0.004651f, 0f, 0.006977f, 0.004651f,
				0f, 0.008527f, 0.012403f, 0.004651f, 0.003876f, 0.003101f, 0.022481f, 0.024031f, 0.00155f, 0.047287f,
				0.009302f, 0.00155f, 0.005426f, 0.017054f
			};
			m_SecondByteStdDev = 0.014156f;
			m_SecondByteMean = 0.010638f;
			m_SecondByteWeight = 0.413467f;
		}
	}
	public class nsBIG5Verifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsBIG5Verifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 286331153;
			m_cclass[1] = 1118481;
			m_cclass[2] = 286331153;
			m_cclass[3] = 286327057;
			m_cclass[4] = 286331153;
			m_cclass[5] = 286331153;
			m_cclass[6] = 286331153;
			m_cclass[7] = 286331153;
			m_cclass[8] = 572662306;
			m_cclass[9] = 572662306;
			m_cclass[10] = 572662306;
			m_cclass[11] = 572662306;
			m_cclass[12] = 572662306;
			m_cclass[13] = 572662306;
			m_cclass[14] = 572662306;
			m_cclass[15] = 304226850;
			m_cclass[16] = 1145324612;
			m_cclass[17] = 1145324612;
			m_cclass[18] = 1145324612;
			m_cclass[19] = 1145324612;
			m_cclass[20] = 858993460;
			m_cclass[21] = 858993459;
			m_cclass[22] = 858993459;
			m_cclass[23] = 858993459;
			m_cclass[24] = 858993459;
			m_cclass[25] = 858993459;
			m_cclass[26] = 858993459;
			m_cclass[27] = 858993459;
			m_cclass[28] = 858993459;
			m_cclass[29] = 858993459;
			m_cclass[30] = 858993459;
			m_cclass[31] = 53687091;
			m_states = new int[3];
			m_states[0] = 286339073;
			m_states[1] = 304226833;
			m_states[2] = 1;
			m_charset = "Big5";
			m_stFactor = 5;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public class nsCP1252Verifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsCP1252Verifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 572662305;
			m_cclass[1] = 2236962;
			m_cclass[2] = 572662306;
			m_cclass[3] = 572654114;
			m_cclass[4] = 572662306;
			m_cclass[5] = 572662306;
			m_cclass[6] = 572662306;
			m_cclass[7] = 572662306;
			m_cclass[8] = 572662306;
			m_cclass[9] = 572662306;
			m_cclass[10] = 572662306;
			m_cclass[11] = 572662306;
			m_cclass[12] = 572662306;
			m_cclass[13] = 572662306;
			m_cclass[14] = 572662306;
			m_cclass[15] = 572662306;
			m_cclass[16] = 572662274;
			m_cclass[17] = 16851234;
			m_cclass[18] = 572662304;
			m_cclass[19] = 285286690;
			m_cclass[20] = 572662306;
			m_cclass[21] = 572662306;
			m_cclass[22] = 572662306;
			m_cclass[23] = 572662306;
			m_cclass[24] = 286331153;
			m_cclass[25] = 286331153;
			m_cclass[26] = 554766609;
			m_cclass[27] = 286331153;
			m_cclass[28] = 286331153;
			m_cclass[29] = 286331153;
			m_cclass[30] = 554766609;
			m_cclass[31] = 286331153;
			m_states = new int[3];
			m_states[0] = 571543601;
			m_states[1] = 340853778;
			m_states[2] = 65;
			m_charset = "windows-1252";
			m_stFactor = 3;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public class nsDetector : nsPSMDetector, nsICharsetDetector
	{
		private nsICharsetDetectionObserver mObserver;

		public nsDetector()
		{
		}

		public nsDetector(int langFlag)
			: base(langFlag)
		{
		}

		public void Init(nsICharsetDetectionObserver aObserver)
		{
			mObserver = aObserver;
		}

		public bool DoIt(byte[] aBuf, int aLen, bool oDontFeedMe)
		{
			if (aBuf == null || oDontFeedMe)
			{
				return false;
			}
			HandleData(aBuf, aLen);
			return mDone;
		}

		public void Done()
		{
			DataEnd();
		}

		public override void Report(string charset)
		{
			if (mObserver != null)
			{
				mObserver.Notify(charset);
			}
		}

		public bool isAscii(byte[] aBuf, int aLen)
		{
			for (int i = 0; i < aLen; i++)
			{
				if ((0x80u & aBuf[i]) != 0)
				{
					return false;
				}
			}
			return true;
		}
	}
	public class nsEUCJPVerifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsEUCJPVerifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 1145324612;
			m_cclass[1] = 1430537284;
			m_cclass[2] = 1145324612;
			m_cclass[3] = 1145328708;
			m_cclass[4] = 1145324612;
			m_cclass[5] = 1145324612;
			m_cclass[6] = 1145324612;
			m_cclass[7] = 1145324612;
			m_cclass[8] = 1145324612;
			m_cclass[9] = 1145324612;
			m_cclass[10] = 1145324612;
			m_cclass[11] = 1145324612;
			m_cclass[12] = 1145324612;
			m_cclass[13] = 1145324612;
			m_cclass[14] = 1145324612;
			m_cclass[15] = 1145324612;
			m_cclass[16] = 1431655765;
			m_cclass[17] = 827675989;
			m_cclass[18] = 1431655765;
			m_cclass[19] = 1431655765;
			m_cclass[20] = 572662309;
			m_cclass[21] = 572662306;
			m_cclass[22] = 572662306;
			m_cclass[23] = 572662306;
			m_cclass[24] = 572662306;
			m_cclass[25] = 572662306;
			m_cclass[26] = 572662306;
			m_cclass[27] = 572662306;
			m_cclass[28] = 0;
			m_cclass[29] = 0;
			m_cclass[30] = 0;
			m_cclass[31] = 1342177280;
			m_states = new int[5];
			m_states[0] = 286282563;
			m_states[1] = 572657937;
			m_states[2] = 286265378;
			m_states[3] = 319885329;
			m_states[4] = 4371;
			m_charset = "EUC-JP";
			m_stFactor = 6;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public class nsEUCKRVerifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsEUCKRVerifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 286331153;
			m_cclass[1] = 1118481;
			m_cclass[2] = 286331153;
			m_cclass[3] = 286327057;
			m_cclass[4] = 286331153;
			m_cclass[5] = 286331153;
			m_cclass[6] = 286331153;
			m_cclass[7] = 286331153;
			m_cclass[8] = 286331153;
			m_cclass[9] = 286331153;
			m_cclass[10] = 286331153;
			m_cclass[11] = 286331153;
			m_cclass[12] = 286331153;
			m_cclass[13] = 286331153;
			m_cclass[14] = 286331153;
			m_cclass[15] = 286331153;
			m_cclass[16] = 0;
			m_cclass[17] = 0;
			m_cclass[18] = 0;
			m_cclass[19] = 0;
			m_cclass[20] = 572662304;
			m_cclass[21] = 858923554;
			m_cclass[22] = 572662306;
			m_cclass[23] = 572662306;
			m_cclass[24] = 572662306;
			m_cclass[25] = 572662322;
			m_cclass[26] = 572662306;
			m_cclass[27] = 572662306;
			m_cclass[28] = 572662306;
			m_cclass[29] = 572662306;
			m_cclass[30] = 572662306;
			m_cclass[31] = 35791394;
			m_states = new int[2];
			m_states[0] = 286331649;
			m_states[1] = 1122850;
			m_charset = "EUC-KR";
			m_stFactor = 4;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public class nsEUCSampler
	{
		private int mTotal;

		private int mThreshold = 200;

		private int mState;

		public int[] mFirstByteCnt = new int[94];

		public int[] mSecondByteCnt = new int[94];

		public float[] mFirstByteFreq = new float[94];

		public float[] mSecondByteFreq = new float[94];

		public nsEUCSampler()
		{
			Reset();
		}

		public void Reset()
		{
			mTotal = 0;
			mState = 0;
			for (int i = 0; i < 94; i++)
			{
				mFirstByteCnt[i] = (mSecondByteCnt[i] = 0);
			}
		}

		public bool EnoughData()
		{
			return mTotal > mThreshold;
		}

		public bool GetSomeData()
		{
			return mTotal > 1;
		}

		public bool Sample(byte[] aIn, int aLen)
		{
			if (mState == 1)
			{
				return false;
			}
			int num = 0;
			int num2 = 0;
			while (num2 < aLen && 1 != mState)
			{
				switch (mState)
				{
				case 0:
					if ((aIn[num] & 0x80u) != 0)
					{
						if (255 == (0xFF & aIn[num]) || 161 > (0xFF & aIn[num]))
						{
							mState = 1;
							break;
						}
						mTotal++;
						mFirstByteCnt[(0xFF & aIn[num]) - 161]++;
						mState = 2;
					}
					break;
				case 2:
					if ((aIn[num] & 0x80u) != 0)
					{
						if (255 == (0xFF & aIn[num]) || 161 > (0xFF & aIn[num]))
						{
							mState = 1;
							break;
						}
						mTotal++;
						mSecondByteCnt[(0xFF & aIn[num]) - 161]++;
						mState = 0;
					}
					else
					{
						mState = 1;
					}
					break;
				default:
					mState = 1;
					break;
				case 1:
					break;
				}
				num2++;
				num++;
			}
			return 1 != mState;
		}

		public void CalFreq()
		{
			for (int i = 0; i < 94; i++)
			{
				mFirstByteFreq[i] = (float)mFirstByteCnt[i] / (float)mTotal;
				mSecondByteFreq[i] = (float)mSecondByteCnt[i] / (float)mTotal;
			}
		}

		public float GetScore(float[] aFirstByteFreq, float aFirstByteWeight, float[] aSecondByteFreq, float aSecondByteWeight)
		{
			return aFirstByteWeight * GetScore(aFirstByteFreq, mFirstByteFreq) + aSecondByteWeight * GetScore(aSecondByteFreq, mSecondByteFreq);
		}

		public float GetScore(float[] array1, float[] array2)
		{
			float num = 0f;
			for (int i = 0; i < 94; i++)
			{
				float num2 = array1[i] - array2[i];
				num += num2 * num2;
			}
			return (float)Math.Sqrt(num) / 94f;
		}
	}
	public abstract class nsEUCStatistics
	{
		public abstract float[] mFirstByteFreq();

		public abstract float mFirstByteStdDev();

		public abstract float mFirstByteMean();

		public abstract float mFirstByteWeight();

		public abstract float[] mSecondByteFreq();

		public abstract float mSecondByteStdDev();

		public abstract float mSecondByteMean();

		public abstract float mSecondByteWeight();

		public nsEUCStatistics()
		{
		}
	}
	public class nsEUCTWVerifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsEUCTWVerifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 572662306;
			m_cclass[1] = 2236962;
			m_cclass[2] = 572662306;
			m_cclass[3] = 572654114;
			m_cclass[4] = 572662306;
			m_cclass[5] = 572662306;
			m_cclass[6] = 572662306;
			m_cclass[7] = 572662306;
			m_cclass[8] = 572662306;
			m_cclass[9] = 572662306;
			m_cclass[10] = 572662306;
			m_cclass[11] = 572662306;
			m_cclass[12] = 572662306;
			m_cclass[13] = 572662306;
			m_cclass[14] = 572662306;
			m_cclass[15] = 572662306;
			m_cclass[16] = 0;
			m_cclass[17] = 100663296;
			m_cclass[18] = 0;
			m_cclass[19] = 0;
			m_cclass[20] = 1145324592;
			m_cclass[21] = 286331221;
			m_cclass[22] = 286331153;
			m_cclass[23] = 286331153;
			m_cclass[24] = 858985233;
			m_cclass[25] = 858993459;
			m_cclass[26] = 858993459;
			m_cclass[27] = 858993459;
			m_cclass[28] = 858993459;
			m_cclass[29] = 858993459;
			m_cclass[30] = 858993459;
			m_cclass[31] = 53687091;
			m_states = new int[6];
			m_states[0] = 338898961;
			m_states[1] = 571543825;
			m_states[2] = 269623842;
			m_states[3] = 286330880;
			m_states[4] = 1052949;
			m_states[5] = 16;
			m_charset = "x-euc-tw";
			m_stFactor = 7;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public class nsGB18030Verifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsGB18030Verifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 286331153;
			m_cclass[1] = 1118481;
			m_cclass[2] = 286331153;
			m_cclass[3] = 286327057;
			m_cclass[4] = 286331153;
			m_cclass[5] = 286331153;
			m_cclass[6] = 858993459;
			m_cclass[7] = 286331187;
			m_cclass[8] = 572662306;
			m_cclass[9] = 572662306;
			m_cclass[10] = 572662306;
			m_cclass[11] = 572662306;
			m_cclass[12] = 572662306;
			m_cclass[13] = 572662306;
			m_cclass[14] = 572662306;
			m_cclass[15] = 1109533218;
			m_cclass[16] = 1717986917;
			m_cclass[17] = 1717986918;
			m_cclass[18] = 1717986918;
			m_cclass[19] = 1717986918;
			m_cclass[20] = 1717986918;
			m_cclass[21] = 1717986918;
			m_cclass[22] = 1717986918;
			m_cclass[23] = 1717986918;
			m_cclass[24] = 1717986918;
			m_cclass[25] = 1717986918;
			m_cclass[26] = 1717986918;
			m_cclass[27] = 1717986918;
			m_cclass[28] = 1717986918;
			m_cclass[29] = 1717986918;
			m_cclass[30] = 1717986918;
			m_cclass[31] = 107374182;
			m_states = new int[6];
			m_states[0] = 318767105;
			m_states[1] = 571543825;
			m_states[2] = 17965602;
			m_states[3] = 286326804;
			m_states[4] = 303109393;
			m_states[5] = 17;
			m_charset = "GB18030";
			m_stFactor = 7;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public class nsGB2312Verifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsGB2312Verifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 286331153;
			m_cclass[1] = 1118481;
			m_cclass[2] = 286331153;
			m_cclass[3] = 286327057;
			m_cclass[4] = 286331153;
			m_cclass[5] = 286331153;
			m_cclass[6] = 286331153;
			m_cclass[7] = 286331153;
			m_cclass[8] = 286331153;
			m_cclass[9] = 286331153;
			m_cclass[10] = 286331153;
			m_cclass[11] = 286331153;
			m_cclass[12] = 286331153;
			m_cclass[13] = 286331153;
			m_cclass[14] = 286331153;
			m_cclass[15] = 286331153;
			m_cclass[16] = 0;
			m_cclass[17] = 0;
			m_cclass[18] = 0;
			m_cclass[19] = 0;
			m_cclass[20] = 572662304;
			m_cclass[21] = 858993442;
			m_cclass[22] = 572662306;
			m_cclass[23] = 572662306;
			m_cclass[24] = 572662306;
			m_cclass[25] = 572662306;
			m_cclass[26] = 572662306;
			m_cclass[27] = 572662306;
			m_cclass[28] = 572662306;
			m_cclass[29] = 572662306;
			m_cclass[30] = 572662306;
			m_cclass[31] = 35791394;
			m_states = new int[2];
			m_states[0] = 286331649;
			m_states[1] = 1122850;
			m_charset = "GB2312";
			m_stFactor = 4;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public class nsHZVerifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsHZVerifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 1;
			m_cclass[1] = 0;
			m_cclass[2] = 0;
			m_cclass[3] = 4096;
			m_cclass[4] = 0;
			m_cclass[5] = 0;
			m_cclass[6] = 0;
			m_cclass[7] = 0;
			m_cclass[8] = 0;
			m_cclass[9] = 0;
			m_cclass[10] = 0;
			m_cclass[11] = 0;
			m_cclass[12] = 0;
			m_cclass[13] = 0;
			m_cclass[14] = 0;
			m_cclass[15] = 38813696;
			m_cclass[16] = 286331153;
			m_cclass[17] = 286331153;
			m_cclass[18] = 286331153;
			m_cclass[19] = 286331153;
			m_cclass[20] = 286331153;
			m_cclass[21] = 286331153;
			m_cclass[22] = 286331153;
			m_cclass[23] = 286331153;
			m_cclass[24] = 286331153;
			m_cclass[25] = 286331153;
			m_cclass[26] = 286331153;
			m_cclass[27] = 286331153;
			m_cclass[28] = 286331153;
			m_cclass[29] = 286331153;
			m_cclass[30] = 286331153;
			m_cclass[31] = 286331153;
			m_states = new int[6];
			m_states[0] = 285213456;
			m_states[1] = 572657937;
			m_states[2] = 335548706;
			m_states[3] = 341120533;
			m_states[4] = 336872468;
			m_states[5] = 36;
			m_charset = "HZ-GB-2312";
			m_stFactor = 6;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public interface nsICharsetDetectionObserver
	{
		void Notify(string charset);
	}
	public interface nsICharsetDetector
	{
		void Init(nsICharsetDetectionObserver observer);

		bool DoIt(byte[] aBuf, int aLen, bool oDontFeedMe);

		void Done();
	}
	public class nsISO2022CNVerifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsISO2022CNVerifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 2;
			m_cclass[1] = 0;
			m_cclass[2] = 0;
			m_cclass[3] = 4096;
			m_cclass[4] = 0;
			m_cclass[5] = 48;
			m_cclass[6] = 0;
			m_cclass[7] = 0;
			m_cclass[8] = 16384;
			m_cclass[9] = 0;
			m_cclass[10] = 0;
			m_cclass[11] = 0;
			m_cclass[12] = 0;
			m_cclass[13] = 0;
			m_cclass[14] = 0;
			m_cclass[15] = 0;
			m_cclass[16] = 572662306;
			m_cclass[17] = 572662306;
			m_cclass[18] = 572662306;
			m_cclass[19] = 572662306;
			m_cclass[20] = 572662306;
			m_cclass[21] = 572662306;
			m_cclass[22] = 572662306;
			m_cclass[23] = 572662306;
			m_cclass[24] = 572662306;
			m_cclass[25] = 572662306;
			m_cclass[26] = 572662306;
			m_cclass[27] = 572662306;
			m_cclass[28] = 572662306;
			m_cclass[29] = 572662306;
			m_cclass[30] = 572662306;
			m_cclass[31] = 572662306;
			m_states = new int[8];
			m_states[0] = 304;
			m_states[1] = 286331152;
			m_states[2] = 572662289;
			m_states[3] = 336663074;
			m_states[4] = 286335249;
			m_states[5] = 286331237;
			m_states[6] = 286335249;
			m_states[7] = 18944273;
			m_charset = "ISO-2022-CN";
			m_stFactor = 9;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public class nsISO2022JPVerifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsISO2022JPVerifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 2;
			m_cclass[1] = 570425344;
			m_cclass[2] = 0;
			m_cclass[3] = 4096;
			m_cclass[4] = 458752;
			m_cclass[5] = 3;
			m_cclass[6] = 0;
			m_cclass[7] = 0;
			m_cclass[8] = 1030;
			m_cclass[9] = 1280;
			m_cclass[10] = 0;
			m_cclass[11] = 0;
			m_cclass[12] = 0;
			m_cclass[13] = 0;
			m_cclass[14] = 0;
			m_cclass[15] = 0;
			m_cclass[16] = 572662306;
			m_cclass[17] = 572662306;
			m_cclass[18] = 572662306;
			m_cclass[19] = 572662306;
			m_cclass[20] = 572662306;
			m_cclass[21] = 572662306;
			m_cclass[22] = 572662306;
			m_cclass[23] = 572662306;
			m_cclass[24] = 572662306;
			m_cclass[25] = 572662306;
			m_cclass[26] = 572662306;
			m_cclass[27] = 572662306;
			m_cclass[28] = 572662306;
			m_cclass[29] = 572662306;
			m_cclass[30] = 572662306;
			m_cclass[31] = 572662306;
			m_states = new int[6];
			m_states[0] = 304;
			m_states[1] = 286331153;
			m_states[2] = 572662306;
			m_states[3] = 1091653905;
			m_states[4] = 303173905;
			m_states[5] = 287445265;
			m_charset = "ISO-2022-JP";
			m_stFactor = 8;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public class nsISO2022KRVerifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsISO2022KRVerifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 2;
			m_cclass[1] = 0;
			m_cclass[2] = 0;
			m_cclass[3] = 4096;
			m_cclass[4] = 196608;
			m_cclass[5] = 64;
			m_cclass[6] = 0;
			m_cclass[7] = 0;
			m_cclass[8] = 20480;
			m_cclass[9] = 0;
			m_cclass[10] = 0;
			m_cclass[11] = 0;
			m_cclass[12] = 0;
			m_cclass[13] = 0;
			m_cclass[14] = 0;
			m_cclass[15] = 0;
			m_cclass[16] = 572662306;
			m_cclass[17] = 572662306;
			m_cclass[18] = 572662306;
			m_cclass[19] = 572662306;
			m_cclass[20] = 572662306;
			m_cclass[21] = 572662306;
			m_cclass[22] = 572662306;
			m_cclass[23] = 572662306;
			m_cclass[24] = 572662306;
			m_cclass[25] = 572662306;
			m_cclass[26] = 572662306;
			m_cclass[27] = 572662306;
			m_cclass[28] = 572662306;
			m_cclass[29] = 572662306;
			m_cclass[30] = 572662306;
			m_cclass[31] = 572662306;
			m_states = new int[5];
			m_states[0] = 285212976;
			m_states[1] = 572657937;
			m_states[2] = 289476898;
			m_states[3] = 286593297;
			m_states[4] = 8465;
			m_charset = "ISO-2022-KR";
			m_stFactor = 6;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public abstract class nsPSMDetector
	{
		public const int ALL = 0;

		public const int JAPANESE = 1;

		public const int CHINESE = 2;

		public const int SIMPLIFIED_CHINESE = 3;

		public const int TRADITIONAL_CHINESE = 4;

		public const int KOREAN = 5;

		public const int NO_OF_LANGUAGES = 6;

		public const int MAX_VERIFIERS = 16;

		private nsVerifier[] mVerifier;

		private nsEUCStatistics[] mStatisticsData;

		private nsEUCSampler mSampler = new nsEUCSampler();

		private byte[] mState = new byte[16];

		private int[] mItemIdx = new int[16];

		private int mItems;

		private int mClassItems;

		protected bool mDone;

		protected bool mRunSampler;

		protected bool mClassRunSampler;

		public nsPSMDetector()
		{
			initVerifiers(0);
			Reset();
		}

		public nsPSMDetector(int langFlag)
		{
			initVerifiers(langFlag);
			Reset();
		}

		public nsPSMDetector(int aItems, nsVerifier[] aVerifierSet, nsEUCStatistics[] aStatisticsSet)
		{
			mClassRunSampler = aStatisticsSet != null;
			mStatisticsData = aStatisticsSet;
			mVerifier = aVerifierSet;
			mClassItems = aItems;
			Reset();
		}

		public void Reset()
		{
			mRunSampler = mClassRunSampler;
			mDone = false;
			mItems = mClassItems;
			for (int i = 0; i < mItems; i++)
			{
				mState[i] = 0;
				mItemIdx[i] = i;
			}
			mSampler.Reset();
		}

		protected void initVerifiers(int currVerSet)
		{
			int num = ((currVerSet >= 0 && currVerSet < 6) ? currVerSet : 0);
			mVerifier = null;
			mStatisticsData = null;
			switch (num)
			{
			case 4:
				mVerifier = new nsVerifier[7]
				{
					new nsUTF8Verifier(),
					new nsBIG5Verifier(),
					new nsISO2022CNVerifier(),
					new nsEUCTWVerifier(),
					new nsCP1252Verifier(),
					new nsUCS2BEVerifier(),
					new nsUCS2LEVerifier()
				};
				mStatisticsData = new nsEUCStatistics[7]
				{
					null,
					new Big5Statistics(),
					null,
					new EUCTWStatistics(),
					null,
					null,
					null
				};
				break;
			case 5:
				mVerifier = new nsVerifier[6]
				{
					new nsUTF8Verifier(),
					new nsEUCKRVerifier(),
					new nsISO2022KRVerifier(),
					new nsCP1252Verifier(),
					new nsUCS2BEVerifier(),
					new nsUCS2LEVerifier()
				};
				break;
			case 3:
				mVerifier = new nsVerifier[8]
				{
					new nsUTF8Verifier(),
					new nsGB2312Verifier(),
					new nsGB18030Verifier(),
					new nsISO2022CNVerifier(),
					new nsHZVerifier(),
					new nsCP1252Verifier(),
					new nsUCS2BEVerifier(),
					new nsUCS2LEVerifier()
				};
				break;
			case 1:
				mVerifier = new nsVerifier[7]
				{
					new nsUTF8Verifier(),
					new nsSJISVerifier(),
					new nsEUCJPVerifier(),
					new nsISO2022JPVerifier(),
					new nsCP1252Verifier(),
					new nsUCS2BEVerifier(),
					new nsUCS2LEVerifier()
				};
				break;
			case 2:
				mVerifier = new nsVerifier[10]
				{
					new nsUTF8Verifier(),
					new nsGB2312Verifier(),
					new nsGB18030Verifier(),
					new nsBIG5Verifier(),
					new nsISO2022CNVerifier(),
					new nsHZVerifier(),
					new nsEUCTWVerifier(),
					new nsCP1252Verifier(),
					new nsUCS2BEVerifier(),
					new nsUCS2LEVerifier()
				};
				mStatisticsData = new nsEUCStatistics[10]
				{
					null,
					new GB2312Statistics(),
					null,
					new Big5Statistics(),
					null,
					null,
					new EUCTWStatistics(),
					null,
					null,
					null
				};
				break;
			case 0:
				mVerifier = new nsVerifier[15]
				{
					new nsUTF8Verifier(),
					new nsSJISVerifier(),
					new nsEUCJPVerifier(),
					new nsISO2022JPVerifier(),
					new nsEUCKRVerifier(),
					new nsISO2022KRVerifier(),
					new nsBIG5Verifier(),
					new nsEUCTWVerifier(),
					new nsGB2312Verifier(),
					new nsGB18030Verifier(),
					new nsISO2022CNVerifier(),
					new nsHZVerifier(),
					new nsCP1252Verifier(),
					new nsUCS2BEVerifier(),
					new nsUCS2LEVerifier()
				};
				mStatisticsData = new nsEUCStatistics[15]
				{
					null,
					null,
					new EUCJPStatistics(),
					null,
					new EUCKRStatistics(),
					null,
					new Big5Statistics(),
					new EUCTWStatistics(),
					new GB2312Statistics(),
					null,
					null,
					null,
					null,
					null,
					null
				};
				break;
			}
			mClassRunSampler = mStatisticsData != null;
			mClassItems = mVerifier.Length;
		}

		public abstract void Report(string charset);

		public bool HandleData(byte[] aBuf, int len)
		{
			for (int i = 0; i < len; i++)
			{
				byte b = aBuf[i];
				int num = 0;
				while (num < mItems)
				{
					byte nextState = nsVerifier.getNextState(mVerifier[mItemIdx[num]], b, mState[num]);
					switch (nextState)
					{
					case 2:
						Report(mVerifier[mItemIdx[num]].charset());
						mDone = true;
						return mDone;
					case 1:
						mItems--;
						if (num < mItems)
						{
							mItemIdx[num] = mItemIdx[mItems];
							mState[num] = mState[mItems];
						}
						break;
					default:
						mState[num++] = nextState;
						break;
					}
				}
				if (mItems <= 1)
				{
					if (1 == mItems)
					{
						Report(mVerifier[mItemIdx[0]].charset());
					}
					mDone = true;
					return mDone;
				}
				int num2 = 0;
				int num3 = 0;
				for (num = 0; num < mItems; num++)
				{
					if (!mVerifier[mItemIdx[num]].isUCS2() && !mVerifier[mItemIdx[num]].isUCS2())
					{
						num2++;
						num3 = num;
					}
				}
				if (1 == num2)
				{
					Report(mVerifier[mItemIdx[num3]].charset());
					mDone = true;
					return mDone;
				}
			}
			if (mRunSampler)
			{
				Sample(aBuf, len);
			}
			return mDone;
		}

		public void DataEnd()
		{
			if (mDone)
			{
				return;
			}
			if (mItems == 2)
			{
				if (mVerifier[mItemIdx[0]].charset() == "GB18030")
				{
					Report(mVerifier[mItemIdx[1]].charset());
					mDone = true;
				}
				else if (mVerifier[mItemIdx[1]].charset() == "GB18030")
				{
					Report(mVerifier[mItemIdx[0]].charset());
					mDone = true;
				}
			}
			if (mRunSampler)
			{
				Sample(null, 0, aLastChance: true);
			}
		}

		public void Sample(byte[] aBuf, int aLen)
		{
			Sample(aBuf, aLen, aLastChance: false);
		}

		public void Sample(byte[] aBuf, int aLen, bool aLastChance)
		{
			int num = 0;
			int num2 = 0;
			for (int i = 0; i < mItems; i++)
			{
				if (mStatisticsData[mItemIdx[i]] != null)
				{
					num2++;
				}
				if (!mVerifier[mItemIdx[i]].isUCS2() && !(mVerifier[mItemIdx[i]].charset() == "GB18030"))
				{
					num++;
				}
			}
			mRunSampler = num2 > 1;
			if (!mRunSampler)
			{
				return;
			}
			mRunSampler = mSampler.Sample(aBuf, aLen);
			if (((!aLastChance || !mSampler.GetSomeData()) && !mSampler.EnoughData()) || num2 != num)
			{
				return;
			}
			mSampler.CalFreq();
			int num3 = -1;
			int num4 = 0;
			float num5 = 0f;
			for (int j = 0; j < mItems; j++)
			{
				if (mStatisticsData[mItemIdx[j]] != null && !(mVerifier[mItemIdx[j]].charset() == "Big5"))
				{
					float score = mSampler.GetScore(mStatisticsData[mItemIdx[j]].mFirstByteFreq(), mStatisticsData[mItemIdx[j]].mFirstByteWeight(), mStatisticsData[mItemIdx[j]].mSecondByteFreq(), mStatisticsData[mItemIdx[j]].mSecondByteWeight());
					if (num4++ == 0 || num5 > score)
					{
						num5 = score;
						num3 = j;
					}
				}
			}
			if (num3 >= 0)
			{
				Report(mVerifier[mItemIdx[num3]].charset());
				mDone = true;
			}
		}

		public string[] getProbableCharsets()
		{
			if (mItems <= 0)
			{
				return new string[1] { "nomatch" };
			}
			string[] array = new string[mItems];
			for (int i = 0; i < mItems; i++)
			{
				array[i] = mVerifier[mItemIdx[i]].charset();
			}
			return array;
		}
	}
	public class nsSJISVerifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsSJISVerifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 286331152;
			m_cclass[1] = 1118481;
			m_cclass[2] = 286331153;
			m_cclass[3] = 286327057;
			m_cclass[4] = 286331153;
			m_cclass[5] = 286331153;
			m_cclass[6] = 286331153;
			m_cclass[7] = 286331153;
			m_cclass[8] = 572662306;
			m_cclass[9] = 572662306;
			m_cclass[10] = 572662306;
			m_cclass[11] = 572662306;
			m_cclass[12] = 572662306;
			m_cclass[13] = 572662306;
			m_cclass[14] = 572662306;
			m_cclass[15] = 304226850;
			m_cclass[16] = 858993459;
			m_cclass[17] = 858993459;
			m_cclass[18] = 858993459;
			m_cclass[19] = 858993459;
			m_cclass[20] = 572662308;
			m_cclass[21] = 572662306;
			m_cclass[22] = 572662306;
			m_cclass[23] = 572662306;
			m_cclass[24] = 572662306;
			m_cclass[25] = 572662306;
			m_cclass[26] = 572662306;
			m_cclass[27] = 572662306;
			m_cclass[28] = 858993459;
			m_cclass[29] = 1145393971;
			m_cclass[30] = 1145324612;
			m_cclass[31] = 279620;
			m_states = new int[3];
			m_states[0] = 286339073;
			m_states[1] = 572657937;
			m_states[2] = 4386;
			m_charset = "Shift_JIS";
			m_stFactor = 6;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public class nsUCS2BEVerifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsUCS2BEVerifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 0;
			m_cclass[1] = 2097408;
			m_cclass[2] = 0;
			m_cclass[3] = 12288;
			m_cclass[4] = 0;
			m_cclass[5] = 3355440;
			m_cclass[6] = 0;
			m_cclass[7] = 0;
			m_cclass[8] = 0;
			m_cclass[9] = 0;
			m_cclass[10] = 0;
			m_cclass[11] = 0;
			m_cclass[12] = 0;
			m_cclass[13] = 0;
			m_cclass[14] = 0;
			m_cclass[15] = 0;
			m_cclass[16] = 0;
			m_cclass[17] = 0;
			m_cclass[18] = 0;
			m_cclass[19] = 0;
			m_cclass[20] = 0;
			m_cclass[21] = 0;
			m_cclass[22] = 0;
			m_cclass[23] = 0;
			m_cclass[24] = 0;
			m_cclass[25] = 0;
			m_cclass[26] = 0;
			m_cclass[27] = 0;
			m_cclass[28] = 0;
			m_cclass[29] = 0;
			m_cclass[30] = 0;
			m_cclass[31] = 1409286144;
			m_states = new int[7];
			m_states[0] = 288626549;
			m_states[1] = 572657937;
			m_states[2] = 291923490;
			m_states[3] = 1713792614;
			m_states[4] = 393569894;
			m_states[5] = 1717659269;
			m_states[6] = 1140326;
			m_charset = "UTF-16BE";
			m_stFactor = 6;
		}

		public override bool isUCS2()
		{
			return true;
		}
	}
	public class nsUCS2LEVerifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsUCS2LEVerifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 0;
			m_cclass[1] = 2097408;
			m_cclass[2] = 0;
			m_cclass[3] = 12288;
			m_cclass[4] = 0;
			m_cclass[5] = 3355440;
			m_cclass[6] = 0;
			m_cclass[7] = 0;
			m_cclass[8] = 0;
			m_cclass[9] = 0;
			m_cclass[10] = 0;
			m_cclass[11] = 0;
			m_cclass[12] = 0;
			m_cclass[13] = 0;
			m_cclass[14] = 0;
			m_cclass[15] = 0;
			m_cclass[16] = 0;
			m_cclass[17] = 0;
			m_cclass[18] = 0;
			m_cclass[19] = 0;
			m_cclass[20] = 0;
			m_cclass[21] = 0;
			m_cclass[22] = 0;
			m_cclass[23] = 0;
			m_cclass[24] = 0;
			m_cclass[25] = 0;
			m_cclass[26] = 0;
			m_cclass[27] = 0;
			m_cclass[28] = 0;
			m_cclass[29] = 0;
			m_cclass[30] = 0;
			m_cclass[31] = 1409286144;
			m_states = new int[7];
			m_states[0] = 288647014;
			m_states[1] = 572657937;
			m_states[2] = 303387938;
			m_states[3] = 1712657749;
			m_states[4] = 357927015;
			m_states[5] = 1427182933;
			m_states[6] = 1381717;
			m_charset = "UTF-16LE";
			m_stFactor = 6;
		}

		public override bool isUCS2()
		{
			return true;
		}
	}
	public class nsUTF8Verifier : nsVerifier
	{
		private static int[] m_cclass;

		private static int[] m_states;

		private static int m_stFactor;

		private static string m_charset;

		public override int[] cclass()
		{
			return m_cclass;
		}

		public override int[] states()
		{
			return m_states;
		}

		public override int stFactor()
		{
			return m_stFactor;
		}

		public override string charset()
		{
			return m_charset;
		}

		public nsUTF8Verifier()
		{
			m_cclass = new int[32];
			m_cclass[0] = 286331153;
			m_cclass[1] = 1118481;
			m_cclass[2] = 286331153;
			m_cclass[3] = 286327057;
			m_cclass[4] = 286331153;
			m_cclass[5] = 286331153;
			m_cclass[6] = 286331153;
			m_cclass[7] = 286331153;
			m_cclass[8] = 286331153;
			m_cclass[9] = 286331153;
			m_cclass[10] = 286331153;
			m_cclass[11] = 286331153;
			m_cclass[12] = 286331153;
			m_cclass[13] = 286331153;
			m_cclass[14] = 286331153;
			m_cclass[15] = 286331153;
			m_cclass[16] = 858989090;
			m_cclass[17] = 1145324612;
			m_cclass[18] = 1145324612;
			m_cclass[19] = 1145324612;
			m_cclass[20] = 1431655765;
			m_cclass[21] = 1431655765;
			m_cclass[22] = 1431655765;
			m_cclass[23] = 1431655765;
			m_cclass[24] = 1717986816;
			m_cclass[25] = 1717986918;
			m_cclass[26] = 1717986918;
			m_cclass[27] = 1717986918;
			m_cclass[28] = -2004318073;
			m_cclass[29] = -2003269496;
			m_cclass[30] = -1145324614;
			m_cclass[31] = 16702940;
			m_states = new int[26];
			m_states[0] = -1408167679;
			m_states[1] = 878082233;
			m_states[2] = 286331153;
			m_states[3] = 286331153;
			m_states[4] = 572662306;
			m_states[5] = 572662306;
			m_states[6] = 290805009;
			m_states[7] = 286331153;
			m_states[8] = 290803985;
			m_states[9] = 286331153;
			m_states[10] = 293041937;
			m_states[11] = 286331153;
			m_states[12] = 293015825;
			m_states[13] = 286331153;
			m_states[14] = 295278865;
			m_states[15] = 286331153;
			m_states[16] = 294719761;
			m_states[17] = 286331153;
			m_states[18] = 298634257;
			m_states[19] = 286331153;
			m_states[20] = 297865489;
			m_states[21] = 286331153;
			m_states[22] = 287099921;
			m_states[23] = 286331153;
			m_states[24] = 285212689;
			m_states[25] = 286331153;
			m_charset = "UTF-8";
			m_stFactor = 16;
		}

		public override bool isUCS2()
		{
			return false;
		}
	}
	public abstract class nsVerifier
	{
		public const byte eStart = 0;

		public const byte eError = 1;

		public const byte eItsMe = 2;

		public const int eidxSft4bits = 3;

		public const int eSftMsk4bits = 7;

		public const int eBitSft4bits = 2;

		public const int eUnitMsk4bits = 15;

		public nsVerifier()
		{
		}

		public abstract string charset();

		public abstract int stFactor();

		public abstract int[] cclass();

		public abstract int[] states();

		public abstract bool isUCS2();

		public static byte getNextState(nsVerifier v, byte b, byte s)
		{
			return (byte)(0xFFu & ((uint)(v.states()[((s * v.stFactor() + ((v.cclass()[(b & 0xFF) >> 3] >> ((b & 7) << 2)) & 0xF)) & 0xFF) >> 3] >> (((s * v.stFactor() + ((v.cclass()[(b & 0xFF) >> 3] >> ((b & 7) << 2)) & 0xF)) & 0xFF & 7) << 2)) & 0xFu));
		}
	}
}
