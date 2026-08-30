using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_BuildQueueAlert
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.BuildQueueAlert); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.BuildQueueAlert)obj;
            //  Serialize QueueTypeId
            s.Write(value.QueueTypeId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.BuildQueueAlert)) as Rts.CnC.Messages.Client.BuildQueueAlert;
            //  Deserialize QueueTypeId
            s.Read(out value.QueueTypeId);

            return value;
        }
        
    }
}
