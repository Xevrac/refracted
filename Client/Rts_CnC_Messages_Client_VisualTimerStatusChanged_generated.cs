using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_VisualTimerStatusChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.VisualTimerStatusChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.VisualTimerStatusChanged)obj;
            //  Serialize TimerStatus
            s.Write(value.TimerStatus);
            //  Serialize DueTimeMilliseconds
            s.Write(value.DueTimeMilliseconds);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.VisualTimerStatusChanged)) as Rts.CnC.Messages.Client.VisualTimerStatusChanged;
            //  Deserialize TimerStatus
            s.Read(out value.TimerStatus);
            //  Deserialize DueTimeMilliseconds
            s.Read(out value.DueTimeMilliseconds);

            return value;
        }
        
    }
}
