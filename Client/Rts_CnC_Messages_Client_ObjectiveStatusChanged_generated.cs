using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ObjectiveStatusChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ObjectiveStatusChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ObjectiveStatusChanged)obj;
            //  Serialize ObjectiveId
            s.Write(value.ObjectiveId);
            //  Serialize ObjectiveStatus
            s.Write(value.ObjectiveStatus);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ObjectiveStatusChanged)) as Rts.CnC.Messages.Client.ObjectiveStatusChanged;
            //  Deserialize ObjectiveId
            s.Read(out value.ObjectiveId);
            //  Deserialize ObjectiveStatus
            s.Read(out value.ObjectiveStatus);

            return value;
        }
        
    }
}
