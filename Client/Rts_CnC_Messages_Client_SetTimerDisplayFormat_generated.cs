using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_SetTimerDisplayFormat
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.SetTimerDisplayFormat); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.SetTimerDisplayFormat)obj;
            //  Serialize TimerFormatHalId
            s.Write(value.TimerFormatHalId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.SetTimerDisplayFormat)) as Rts.CnC.Messages.Client.SetTimerDisplayFormat;
            //  Deserialize TimerFormatHalId
            s.Read(out value.TimerFormatHalId);

            return value;
        }
        
    }
}
